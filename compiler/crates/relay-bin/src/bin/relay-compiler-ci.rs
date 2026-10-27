/*
 * Copyright (c) Atlassian and its affiliates.
 *
 * This source code is licensed under the MIT license found in the
 * LICENSE file in the root directory of this source tree.
 */

use std::fs;
use std::fs::File;
use std::io;
use std::io::BufReader;
use std::path::Component;
use std::path::Path;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;

use clap::Parser;
use common::ConsoleLogger;
use relay_compiler::ArtifactValidationWriter;
use relay_compiler::FileSourceKind;
use relay_compiler::LocalPersister;
use relay_compiler::OperationPersister;
use relay_compiler::PersistConfig;
use relay_compiler::RemotePersister;
use relay_compiler::compiler::Compiler;
use relay_compiler::compiler_state::CompilerState;
use relay_compiler::config::Config;
use serde::Deserialize;
use simplelog::LevelFilter;
use simplelog::SimpleLogger;

#[derive(Parser)]
#[clap(about = "Experimental one-shot Relay checkpoint runner", version)]
struct Args {
    /// Evaluated Relay configuration. No custom project selection or watch mode.
    config: PathBuf,
    /// Validate existing artifacts instead of generating them.
    #[clap(long)]
    validate: bool,
    /// Replace the working checkpoint after success; reuse it for eligible deltas.
    #[clap(long)]
    state_file: Option<PathBuf>,
    /// Complete root-relative JSON delta; every entry requires name and exists.
    #[clap(long, requires = "state_file")]
    changed_files: Option<PathBuf>,
}

#[derive(Deserialize)]
struct ChangedFile {
    name: PathBuf,
    // Require the caller to resolve existence against config.root, not process cwd.
    #[serde(rename = "exists")]
    _exists: bool,
}

#[tokio::main]
async fn main() {
    SimpleLogger::init(LevelFilter::Info, simplelog::Config::default())
        .expect("Unable to initialize compiler logging");
    if let Err(error) = run(Args::parse()).await {
        eprintln!("{error}");
        std::process::exit(1);
    }
}

async fn run(args: Args) -> Result<(), Box<dyn std::error::Error>> {
    let mut config = Config::load(args.config)?;
    if args.validate {
        config.artifact_writer = Box::<ArtifactValidationWriter>::default();
    }
    // Match the existing Atlassian CLI's extensions and persistence semantics.
    config.generate_extra_artifacts = Some(Box::new(persisted_mocks::generate_persisted_mocks));
    config.create_operation_persister = Some(Box::new(|project| {
        project
            .persist
            .as_ref()
            .map(|persist| -> Box<dyn OperationPersister + Send + Sync> {
                match persist {
                    PersistConfig::Remote(value) => Box::new(RemotePersister::new(value.clone())),
                    PersistConfig::Local(value) => Box::new(LocalPersister::new(value.clone())),
                }
            })
    }));
    config.file_source_config = if let Some(changes) = args.changed_files {
        // Input-contract checks belong to this caller, not shared file-source policy.
        // Inputs must remain immutable between this check and native compilation.
        let files: Vec<ChangedFile> =
            serde_json::from_reader(BufReader::new(File::open(&changes)?))?;
        let output_dirs = config.get_output_dir_paths();
        // If an output root is not normalized/root-relative, do not guess how
        // it matches the delta: conservatively choose full compilation.
        let mut generated_changed = output_dirs.iter().any(|root| {
            root.components()
                .any(|part| !matches!(part, Component::Normal(_)))
        });
        for file in files {
            if file.name.as_os_str().is_empty()
                || file.name.to_string_lossy().contains(['\\', ':'])
                || file.name.components().any(|part| {
                    matches!(
                        part,
                        Component::Prefix(_)
                            | Component::RootDir
                            | Component::ParentDir
                            | Component::CurDir
                    )
                })
            {
                return Err(format!(
                    "Invalid root-relative changed-file path: {}",
                    file.name.display()
                )
                .into());
            }
            // Be conservative for all files under generated roots, including
            // deletions and other tools' outputs. Do not reproduce ownership logic.
            generated_changed |= file
                .name
                .components()
                .any(|part| matches!(part, Component::Normal(name) if name == "__generated__"))
                || output_dirs.iter().any(|root| file.name.starts_with(root));
        }
        if generated_changed {
            eprintln!("[relay-ci] selected=full-generated-artifacts");
            FileSourceKind::WalkDir
        } else {
            eprintln!("[relay-ci] selected=external");
            config.load_saved_state_file = args.state_file.clone();
            FileSourceKind::External(changes)
        }
    } else {
        eprintln!("[relay-ci] selected=full");
        FileSourceKind::WalkDir
    };
    // No custom restore, invalidation, transforms, dependency traversal or retry.
    // External-state setup errors retain upstream's Watchman recovery behavior.
    let state = Compiler::new(Arc::new(config), Arc::new(ConsoleLogger))
        .compile()
        .await?;
    if let Some(path) = args.state_file {
        save_state(&state, &path)?;
    }
    Ok(())
}

static NEXT_TEMP: AtomicUsize = AtomicUsize::new(0);

// Rename is atomic, but native serialization can discard buffered-write errors.
// This is a disposable cache checkpoint, not a verified/durable publication.
fn save_state(state: &CompilerState, path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let write_error = |err: io::Error| {
        io::Error::other(format!(
            "Unable to save checkpoint {}: {err}",
            path.display()
        ))
    };
    let parent = path.parent().unwrap_or_else(|| Path::new("."));
    let directory = loop {
        let directory = parent.join(format!(
            ".relay-state-{}-{}",
            std::process::id(),
            NEXT_TEMP.fetch_add(1, Ordering::Relaxed)
        ));
        match fs::create_dir(&directory) {
            Ok(()) => break directory,
            Err(err) if err.kind() == io::ErrorKind::AlreadyExists => continue,
            Err(err) => return Err(write_error(err).into()),
        }
    };
    let temporary = directory.join("state.bin");
    let result = (|| {
        state.serialize_to_file(&temporary)?;
        fs::rename(&temporary, path).map_err(write_error)?;
        Ok(())
    })();
    // Cleanup must not turn successful checkpoint replacement into a failed run.
    let _ = fs::remove_file(&temporary);
    let _ = fs::remove_dir(&directory);
    result
}
