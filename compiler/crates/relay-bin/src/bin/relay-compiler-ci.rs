/*
 * Copyright (c) Atlassian and its affiliates.
 *
 * This source code is licensed under the MIT license found in the
 * LICENSE file in the root directory of this source tree.
 */

//! Experimental CI-only validation: restore, validate, and refresh compiler state.
//! Inputs must match HEAD and remain immutable for the entire invocation.
//! Uses the existing compiler library without changing the normal CLI or compiler.
//! Cache transport requires a trusted store; production authentication is not yet implemented.
use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::fs;
use std::future::Future;
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;
use std::pin::Pin;
use std::process::Command;
use std::process::Stdio;
use std::sync::Arc;
use std::time::Instant;
use std::time::SystemTime;
use std::time::UNIX_EPOCH;

use clap::Parser;
use common::ConsoleLogger;
use common::PerfLogger;
use relay_compiler::ArtifactSourceKey;
use relay_compiler::ArtifactValidationWriter;
use relay_compiler::FileCategorizer;
use relay_compiler::FileGroup;
use relay_compiler::FileSource;
use relay_compiler::FileSourceKind;
use relay_compiler::GraphQLAsts;
use relay_compiler::OperationPersister;
use relay_compiler::compiler::Compiler;
use relay_compiler::compiler_state::ArtifactMapKind;
use relay_compiler::compiler_state::CompilerState;
use relay_compiler::compiler_state::GraphQLSources;
use relay_compiler::config::ArtifactForPersister;
use relay_compiler::config::Config;
use relay_compiler::config::PersistId;
use relay_compiler::config::PersistResult;
use relay_compiler::errors::Error;
use serde::Deserialize;
use serde::Serialize;
use serde_json::Value;
use serde_json::json;
use simplelog::ColorChoice;
use simplelog::ConfigBuilder;
use simplelog::LevelFilter;
use simplelog::TermLogger;
use simplelog::TerminalMode;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[derive(Parser)]
struct Args {
    #[arg(default_value = "relay.config.js")]
    config: PathBuf,
    #[arg(long, required = true)]
    validate: bool,
    /// Trusted, repository-scoped object store supporting GET and PUT.
    #[arg(long, env = "RELAY_CI_CACHE_URL")]
    cache_url: Option<String>,
    /// Benchmark/reference full validation, without cache reads or writes.
    #[arg(long)]
    no_cache: bool,
    #[arg(long)]
    report: Option<PathBuf>,
}

struct Scratch(PathBuf);
impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

#[derive(Serialize, Deserialize)]
struct Manifest {
    version: u32,
    revision: String,
    key: String,
    state_hash: String,
    state_bytes: u64,
}

#[derive(Default, Serialize)]
struct Report {
    compatibility_key: String,
    mode: String,
    reason: String,
    revision: String,
    snapshot_revision: Option<String>,
    changed_files: usize,
    owned_artifact_changes: usize,
    downloaded_bytes: u64,
    uploaded_bytes: u64,
    published: bool,
    validation_passed: bool,
    phases: BTreeMap<String, f64>,
    total_seconds: f64,
}

// A valid checked-in artifact reuses its existing persisted ID without calling
// this method. If Relay needs a NEW ID, CI must fail and request regeneration,
// not register queries (or fabricate IDs) during a validation-only run.
struct ValidationPersister;
impl OperationPersister for ValidationPersister {
    fn persist_artifact<'life0, 'async_trait>(
        &'life0 self,
        artifact: ArtifactForPersister,
    ) -> Pin<Box<dyn Future<Output = PersistResult<PersistId>> + Send + 'async_trait>>
    where
        'life0: 'async_trait,
        Self: 'async_trait,
    {
        Box::pin(async move {
            Err(std::io::Error::other(format!(
                "Artifact regeneration required for {}: validation never registers new persisted queries",
                artifact.relative_path.display()
            ))
            .into())
        })
    }
}

fn git(root: &Path, args: &[&str]) -> Result<Vec<u8>> {
    let output = Command::new("git")
        .arg("-C")
        .arg(root)
        .args(args)
        .output()?;
    if !output.status.success() {
        return Err(format!(
            "git {}: {}",
            args.join(" "),
            String::from_utf8_lossy(&output.stderr)
        )
        .into());
    }
    Ok(output.stdout)
}

fn fingerprint(root: &Path, bytes: &[u8]) -> Result<String> {
    let mut child = Command::new("git")
        .arg("-C")
        .arg(root)
        .args(["hash-object", "--stdin"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()?;
    child
        .stdin
        .take()
        .ok_or("Missing hash stdin")?
        .write_all(bytes)?;
    let output = child.wait_with_output()?;
    if !output.status.success() {
        return Err("git hash-object failed".into());
    }
    Ok(String::from_utf8(output.stdout)?.trim().to_owned())
}

fn paths(bytes: Vec<u8>) -> Result<Vec<String>> {
    String::from_utf8(bytes)?
        .split('\0')
        .filter(|s| !s.is_empty())
        .map(|s| {
            let path = Path::new(s);
            if path.is_absolute()
                || path
                    .components()
                    .any(|part| matches!(part, std::path::Component::ParentDir))
            {
                Err(format!("Unsafe Git delta path: {s}").into())
            } else {
                Ok(s.to_owned())
            }
        })
        .collect()
}

fn git_inputs(root: &Path, args: &[&str], inputs: &[String]) -> Result<Vec<u8>> {
    let mut scoped = args.to_vec();
    scoped.push("--");
    scoped.extend(inputs.iter().map(String::as_str));
    git(root, &scoped)
}

fn delta(root: &Path, revision: &str, inputs: &[String]) -> Result<Vec<Value>> {
    // CI-only: the materialized inputs must match HEAD and stay immutable.
    // Compare committed trees; never refresh the index against the worktree.
    // --no-renames represents moves as a deletion plus addition.
    git(
        root,
        &["rev-parse", "--verify", &format!("{revision}^{{commit}}")],
    )?;
    let names: BTreeSet<String> = paths(git_inputs(
        root,
        &[
            "diff",
            "--no-renames",
            "--name-only",
            "-z",
            revision,
            "HEAD",
        ],
        inputs,
    )?)?
    .into_iter()
    .collect();
    Ok(names
        .into_iter()
        .map(|name| json!({"exists": root.join(&name).is_file(), "name": name}))
        .collect())
}

// Relay's non-Watchman FileFilter excludes default __generated__ directories,
// even for an external delta. Permit known Relay-owned artifact changes so a
// pure artifact deletion/edit cannot silently pass. Never claim unrelated
// generated outputs: only saved ownership or explicit project outputs qualify.
fn admit_artifact_changes(
    cfg: &mut Config,
    state: &CompilerState,
    changes: Vec<Value>,
) -> Result<(Vec<Value>, usize)> {
    let mut owned = BTreeSet::new();
    for mapping in state.artifacts.0.values() {
        if let ArtifactMapKind::Mapping(mapping) = mapping.as_ref() {
            for entry in mapping.0.iter() {
                owned.extend(entry.value().iter().map(|record| record.path.clone()));
            }
        }
    }
    let categorizer = FileCategorizer::from_config(cfg);
    let output_roots = cfg.get_output_dir_paths();
    let mut admitted = 0;
    let filtered = changes
        .into_iter()
        .filter(|change| {
            let Some(name) = change["name"].as_str() else {
                return false;
            };
            let path = Path::new(name);
            if matches!(
                categorizer.categorize(path, cfg),
                Ok(FileGroup::Generated { .. })
            ) {
                if owned.contains(path) || output_roots.iter().any(|root| path.starts_with(root)) {
                    admitted += 1;
                    true
                } else {
                    false
                }
            } else {
                true
            }
        })
        .collect();
    if admitted != 0 {
        cfg.excludes
            .retain(|pattern| pattern != "**/__generated__/**");
    }
    Ok((filtered, admitted))
}

// A saved ownership map can flag orphaned outputs after a source deletion,
// while the stock full CLI excludes default generated directories and may leave
// those outputs alone. Keep cache availability from changing validation results:
// source removals/renames use the stock full path in this first CI prototype.
fn source_removal(cfg: &Config, changes: &[Value]) -> bool {
    let categorizer = FileCategorizer::from_config(cfg);
    changes.iter().any(|change| {
        change["exists"].as_bool() == Some(false)
            && change["name"].as_str().is_some_and(|name| {
                matches!(
                    categorizer.categorize(Path::new(name), cfg),
                    Ok(FileGroup::Source { .. }
                        | FileGroup::Schema { .. }
                        | FileGroup::Extension { .. })
                )
            })
    })
}

fn removed_definitions_or_non_graphql_changes(
    state: &CompilerState,
    cfg: &Config,
    changes: &[Value],
) -> Result<bool> {
    if state.schemas.values().any(|s| !s.pending.is_empty())
        || state.extensions.values().any(|s| !s.pending.is_empty())
        || state.docblocks.values().any(|s| !s.pending.is_empty())
        || state.full_sources.values().any(|s| !s.pending.is_empty())
    {
        return Ok(true);
    }
    let changed_paths: BTreeSet<PathBuf> = changes
        .iter()
        .filter_map(|change| change["name"].as_str())
        .map(PathBuf::from)
        .collect();
    for mapping in state.artifacts.0.values() {
        if let ArtifactMapKind::Mapping(mapping) = mapping.as_ref()
            && mapping.0.iter().any(|entry| {
                !matches!(entry.key(), ArtifactSourceKey::ExecutableDefinition(_))
                    && entry
                        .value()
                        .iter()
                        .any(|record| changed_paths.contains(&record.path))
            })
        {
            return Ok(true);
        }
    }
    for (project, sources) in &state.graphql_sources {
        if sources.pending.is_empty() {
            continue;
        }
        // Ask Relay's parser, not a regex, whether definitions disappeared.
        // Only clone/parse changed files and their previous versions.
        let mut changed = GraphQLSources {
            pending: sources.pending.clone(),
            ..Default::default()
        };
        for file in changed.pending.keys() {
            if let Some(old) = sources.processed.get(file) {
                changed.processed.insert(file.clone(), old.clone());
            }
        }
        let asts = GraphQLAsts::from_graphql_sources(&changed, None, &cfg.projects[project])?;
        if !asts.removed_definition_names.is_empty() {
            return Ok(true);
        }
    }
    Ok(false)
}

fn transfer(url: &str, file: &Path, upload: bool) -> Result<()> {
    let mut cmd = Command::new("curl");
    cmd.args([
        "--fail",
        "--silent",
        "--show-error",
        "--connect-timeout",
        "5",
        "--max-time",
        "30",
        "--max-filesize",
        "268435456",
    ]);
    if upload {
        cmd.arg("--upload-file")
            .arg(file)
            .arg("--output")
            .arg("/dev/null");
    } else {
        cmd.arg("--output").arg(file);
    }
    let output = cmd.arg(url).output()?;
    if !output.status.success() {
        return Err(format!(
            "Cache {} failed: {}",
            if upload { "upload" } else { "restore" },
            String::from_utf8_lossy(&output.stderr).trim()
        )
        .into());
    }
    Ok(())
}

fn config(path: &Path, version: &str) -> Result<Config> {
    let mut cfg = Config::load(path.to_owned())?;
    // The hash covers the complete evaluated config, with only transport auth,
    // build attribution, and the checkout's absolute prefix normalized below.
    cfg.saved_state_version = version.to_owned();
    cfg.artifact_writer = Box::<ArtifactValidationWriter>::default();
    cfg.file_source_config = FileSourceKind::WalkDir;
    cfg.generate_extra_artifacts = Some(Box::new(persisted_mocks::generate_persisted_mocks));
    cfg.create_operation_persister = Some(Box::new(|project| {
        project
            .persist
            .as_ref()
            .map(|_| -> Box<dyn OperationPersister + Send + Sync> { Box::new(ValidationPersister) })
    }));
    Ok(cfg)
}

async fn run(args: &Args, scratch: &Path, report: &mut Report) -> Result<()> {
    if std::env::var_os("RELAY_COMPILER_IGNORE_SAVED_STATE_VERSION").is_some() {
        return Err("Saved-state version checks must not be disabled".into());
    }
    let start = Instant::now();
    let root = PathBuf::from(
        String::from_utf8(git(Path::new("."), &["rev-parse", "--show-toplevel"])?)?.trim(),
    );
    report.revision = String::from_utf8(git(&root, &["rev-parse", "HEAD"])?)?
        .trim()
        .to_owned();
    let binary_hash = fingerprint(&root, &fs::read(std::env::current_exe()?)?)?;
    let sparse_path = String::from_utf8(git(
        &root,
        &["rev-parse", "--git-path", "info/sparse-checkout"],
    )?)?;
    let sparse = fs::read(root.join(sparse_path.trim())).unwrap_or_default();
    report
        .phases
        .insert("identity".into(), start.elapsed().as_secs_f64());

    let start = Instant::now();
    let config_path = fs::canonicalize(&args.config)?;
    // Evaluate once, without running AFM's local authentication command. Preserve
    // every codegen setting and endpoint. No credentials are needed by this binary.
    let script = r#"
const path = require('node:path');
const cfg = require(process.argv[1]);
for (const project of Object.values(cfg.projects || { default: cfg })) {
  if (project.persist?.headers) {
    for (const key of Object.keys(project.persist.headers)) {
      if (key.toLowerCase() === 'authorization') delete project.persist.headers[key];
    }
  }
  if (project.persist?.params) delete project.persist.params.bitbucketBuildNumber;
}
cfg.root = path.resolve(path.dirname(process.argv[1]), cfg.root || '.');
cfg.noSourceControl = true;
process.stdout.write(JSON.stringify(cfg));
"#;
    let output = Command::new("node")
        .arg("-e")
        .arg(script)
        .arg(&config_path)
        .env("TEST", "1")
        .output()?;
    if !output.status.success() {
        return Err(format!(
            "Config evaluation failed: {}",
            String::from_utf8_lossy(&output.stderr)
        )
        .into());
    }
    let mut value: Value = serde_json::from_slice(&output.stdout)?;
    let configured_root = fs::canonicalize(value["root"].as_str().ok_or("Missing config root")?)?;
    if configured_root != fs::canonicalize(&root)? {
        return Err("Prototype requires Relay root == Git worktree root".into());
    }
    value["root"] = json!("<git-worktree-root>");
    let config_hash = fingerprint(&root, &serde_json::to_vec(&value)?)?;
    let version = format!("relay-ci-v1-{config_hash}");
    let key = fingerprint(
        &root,
        &serde_json::to_vec(
            &json!({"adapter": 1, "binary": binary_hash, "config": config_hash, "sparse": sparse}),
        )?,
    )?;
    report.compatibility_key = key.clone();
    value["root"] = json!(configured_root);
    let evaluated_config = scratch.join("config.json");
    fs::write(&evaluated_config, serde_json::to_vec(&value)?)?;
    let mut cfg = Arc::new(config(&evaluated_config, &version)?);
    // Cover the compiler's entire input/output universe, including schemas,
    // extensions and extra mocks. Unrelated AFM products/tooling need no stat scan.
    let inputs: Vec<String> = cfg
        .get_all_roots()
        .iter()
        .map(|path| {
            let relative = if path.is_absolute() {
                path.strip_prefix(&root)
                    .map_err(|_| "Relay input outside Git root")?
            } else {
                path.as_path()
            };
            Ok(format!(":(literal){}", relative.display()))
        })
        .collect::<Result<_>>()?;
    report
        .phases
        .insert("config".into(), start.elapsed().as_secs_f64());
    let cache = if args.no_cache {
        None
    } else {
        args.cache_url
            .as_ref()
            .map(|base| format!("{}/{key}", base.trim_end_matches('/')))
    };

    // The CI checkout contract replaces a costly filesystem cleanliness scan.
    // No preceding/concurrent step may rewrite tracked Relay inputs or create
    // required untracked inputs. Ordinary local commands use the normal compiler.
    let state_path = scratch.join("state.bin");
    let manifest_path = scratch.join("manifest.json");
    let delta_path = scratch.join("changes.json");
    let restore_start = Instant::now();
    let restored: Result<Manifest> = (|| {
        let prefix = cache.as_ref().ok_or("Cache disabled or not configured")?;
        transfer(&format!("{prefix}/latest.json"), &manifest_path, false)?;
        let manifest: Manifest = serde_json::from_slice(&fs::read(&manifest_path)?)?;
        if manifest.version != 1
            || manifest.key != key
            || ![40, 64].contains(&manifest.revision.len())
            || !manifest.revision.bytes().all(|c| c.is_ascii_hexdigit())
            || ![40, 64].contains(&manifest.state_hash.len())
            || !manifest.state_hash.bytes().all(|c| c.is_ascii_hexdigit())
            || manifest.state_bytes > 268435456
        {
            return Err("Incompatible or invalid manifest".into());
        }
        transfer(
            &format!("{prefix}/{}-{}.bin", manifest.revision, manifest.state_hash),
            &state_path,
            false,
        )?;
        let bytes = fs::read(&state_path)?;
        report.downloaded_bytes = bytes.len() as u64;
        if bytes.len() as u64 != manifest.state_bytes
            || fingerprint(&root, &bytes)? != manifest.state_hash
        {
            return Err("Snapshot checksum/length mismatch".into());
        }
        Ok(manifest)
    })();
    report.phases.insert(
        "cache_restore".into(),
        restore_start.elapsed().as_secs_f64(),
    );

    report.mode = "full".into();
    let logger = Arc::new(ConsoleLogger);
    match restored {
        Ok(manifest) => {
            report.snapshot_revision = Some(manifest.revision.clone());
            let start = Instant::now();
            let changes = delta(&root, &manifest.revision, &inputs);
            report
                .phases
                .insert("git_delta".into(), start.elapsed().as_secs_f64());
            match changes {
                Ok(changes) if source_removal(&cfg, &changes) => {
                    report.changed_files = changes.len();
                    report.reason =
                        "Source removal/rename: full validation for stock CLI parity".into();
                }
                Ok(changes) => {
                    report.changed_files = changes.len();
                    fs::write(&delta_path, serde_json::to_vec(&changes)?)?;
                    let mut external_cfg = config(&evaluated_config, &version)?;
                    external_cfg.load_saved_state_file = Some(state_path.clone());
                    external_cfg.file_source_config = FileSourceKind::External(delta_path.clone());
                    let mut external_cfg = Arc::new(external_cfg);
                    let start = Instant::now();
                    let event = logger.create_event("ci_snapshot_preflight");
                    let source = FileSource::connect(&external_cfg, &event).await?;
                    if let FileSource::External(external) = source {
                        let preflight = external.create_compiler_state(logger.as_ref());
                        drop(external);
                        match preflight {
                            Ok(state)
                                if removed_definitions_or_non_graphql_changes(
                                    &state,
                                    &external_cfg,
                                    &changes,
                                )? =>
                            {
                                report.reason = "Removed definitions/schema/resolver change: full validation for stock CLI parity".into();
                            }
                            Ok(state) => {
                                let mutable = Arc::get_mut(&mut external_cfg)
                                    .ok_or("Unexpected shared config during preflight")?;
                                let (changes, admitted) =
                                    admit_artifact_changes(mutable, &state, changes)?;
                                fs::write(&delta_path, serde_json::to_vec(&changes)?)?;
                                report.owned_artifact_changes = admitted;
                                cfg = external_cfg;
                                report.mode = "incremental".into();
                            }
                            Err(error) => {
                                report.reason = format!("Saved-state preflight failed: {error}")
                            }
                        }
                    }
                    report
                        .phases
                        .insert("preflight".into(), start.elapsed().as_secs_f64());
                }
                Err(error) => report.reason = format!("Delta unavailable: {error}"),
            }
        }
        Err(error) => report.reason = error.to_string(),
    }
    eprintln!(
        "[relay-ci] {} validation; {} changed files; {}",
        report.mode, report.changed_files, report.reason
    );
    let start = Instant::now();
    let result = Compiler::new(cfg, Arc::clone(&logger)).compile().await;
    let state = match result {
        Err(Error::Watchman { .. }) => {
            // Only External's failure path can request Watchman here. Retry this
            // infrastructure failure, never GraphQL/PQ/artifact-validation errors.
            report.mode = "full".into();
            report.reason = "External inputs raced preflight; full filesystem retry".into();
            Compiler::new(Arc::new(config(&evaluated_config, &version)?), logger)
                .compile()
                .await
        }
        result => result,
    };
    report
        .phases
        .insert("compile".into(), start.elapsed().as_secs_f64());
    let state = state?;
    report.validation_passed = true;

    // A successful validation returns complete compiler state. It is a valid
    // producer too: no generation-only job or mode is necessary.
    if let Some(prefix) = cache {
        let publish: Result<()> = (|| {
            // Inputs must match HEAD for this entire CI invocation; only the
            // cheap revision guard is needed. No full worktree/index refresh.
            if String::from_utf8(git(&root, &["rev-parse", "HEAD"])?)?.trim() != report.revision {
                return Err("HEAD changed during validation: skip snapshot publication".into());
            }
            let start = Instant::now();
            state.serialize_to_file(&state_path)?;
            let bytes = fs::read(&state_path)?;
            let manifest = Manifest {
                version: 1,
                revision: report.revision.clone(),
                key,
                state_hash: fingerprint(&root, &bytes)?,
                state_bytes: bytes.len() as u64,
            };
            fs::write(&manifest_path, serde_json::to_vec_pretty(&manifest)?)?;
            report.phases.insert(
                "serialize_and_checksum".into(),
                start.elapsed().as_secs_f64(),
            );
            let start = Instant::now();
            // Immutable payload first, pointer last: readers never observe a
            // manifest for a payload that has not finished uploading.
            transfer(
                &format!("{prefix}/{}-{}.bin", manifest.revision, manifest.state_hash),
                &state_path,
                true,
            )?;
            transfer(&format!("{prefix}/latest.json"), &manifest_path, true)?;
            report
                .phases
                .insert("cache_publish".into(), start.elapsed().as_secs_f64());
            report.uploaded_bytes = manifest.state_bytes + fs::metadata(&manifest_path)?.len();
            report.published = true;
            Ok(())
        })();
        if let Err(error) = publish {
            eprintln!("[relay-ci] Validation passed; cache update skipped: {error}");
        }
    }
    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    TermLogger::init(
        LevelFilter::Info,
        ConfigBuilder::new()
            .set_time_level(LevelFilter::Off)
            .build(),
        TerminalMode::Stderr,
        ColorChoice::Auto,
    )?;
    let scratch = Scratch(std::env::temp_dir().join(format!(
        "relay-ci-{}-{}",
        std::process::id(),
        SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos()
    )));
    fs::create_dir(&scratch.0)?;
    let start = Instant::now();
    let mut report = Report::default();
    let result = run(&args, &scratch.0, &mut report).await;
    report.total_seconds = start.elapsed().as_secs_f64();
    if let Some(path) = args.report {
        fs::write(path, serde_json::to_vec_pretty(&report)?)?;
    }
    eprintln!("[relay-ci-report] {}", serde_json::to_string(&report)?);
    result
}

#[cfg(test)]
mod tests {
    use relay_compiler::Artifact;
    use relay_compiler::ArtifactContent;
    use relay_compiler::ArtifactSourceKey;

    use super::*;

    #[test]
    fn nul_paths_preserve_newlines_and_reject_traversal() {
        assert_eq!(
            paths(b"src/a\nfile.ts\0src/b.ts\0".to_vec()).unwrap(),
            ["src/a\nfile.ts", "src/b.ts"]
        );
        assert!(paths(b"/absolute.ts\0".to_vec()).is_err());
        assert!(paths(b"src/../escape.ts\0".to_vec()).is_err());
        assert!(paths(vec![255, 0]).is_err());
    }

    #[test]
    fn committed_delta_does_not_scan_worktree_only_changes() {
        let unique = format!(
            "relay-ci-tree-test-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        );
        let scratch = Scratch(std::env::temp_dir().join(unique));
        fs::create_dir(&scratch.0).unwrap();
        fs::create_dir(scratch.0.join("src")).unwrap();
        git(&scratch.0, &["init", "-q"]).unwrap();
        git(&scratch.0, &["config", "user.name", "test"]).unwrap();
        git(
            &scratch.0,
            &["config", "user.email", "test@example.invalid"],
        )
        .unwrap();
        fs::write(scratch.0.join("src/committed.ts"), "base").unwrap();
        fs::write(scratch.0.join("src/local.ts"), "base").unwrap();
        git(&scratch.0, &["add", "."]).unwrap();
        git(&scratch.0, &["commit", "-qm", "base fixture"]).unwrap();
        let base = String::from_utf8(git(&scratch.0, &["rev-parse", "HEAD"]).unwrap()).unwrap();
        fs::write(scratch.0.join("src/committed.ts"), "committed").unwrap();
        git(&scratch.0, &["add", "."]).unwrap();
        git(&scratch.0, &["commit", "-qm", "committed change"]).unwrap();
        fs::write(scratch.0.join("src/local.ts"), "unstaged local change").unwrap();
        fs::write(scratch.0.join("src/untracked.ts"), "untracked local input").unwrap();
        let changes = delta(&scratch.0, base.trim(), &[":(literal)src".into()]).unwrap();
        assert_eq!(
            changes,
            vec![json!({"name": "src/committed.ts", "exists": true})]
        );
    }

    #[test]
    fn validation_flag_is_mandatory() {
        assert!(Args::try_parse_from(["relay-compiler-ci"]).is_err());
        assert!(Args::try_parse_from(["relay-compiler-ci", "--validate"]).is_ok());
    }

    #[test]
    fn artifact_admission_is_ownership_scoped() {
        let mut cfg = Config::from_string_for_test(
            r#"{
          "sources": {"src": "app"},
          "projects": {"app": {"schema": "schema.graphql", "language": "typescript"}}
        }"#,
        )
        .unwrap();
        let project = *cfg.projects.keys().next().unwrap();
        let mut state = CompilerState::default();
        let mapping = ArtifactMapKind::Mapping(Default::default());
        if let ArtifactMapKind::Mapping(map) = &mapping {
            map.insert(Artifact {
                artifact_source_keys: vec![ArtifactSourceKey::Schema()],
                path: PathBuf::from("src/__generated__/Owned.graphql.ts"),
                content: ArtifactContent::Generic {
                    content: Vec::new(),
                },
                source_file: common::SourceLocationKey::Generated,
            });
        }
        state.artifacts.0.insert(project, Arc::new(mapping));
        let changes = vec![
            json!({"name": "src/__generated__/Owned.graphql.ts", "exists": false}),
            json!({"name": "src/__generated__/OtherTool.ts", "exists": true}),
            json!({"name": "src/App.ts", "exists": true}),
        ];
        let (filtered, admitted) = admit_artifact_changes(&mut cfg, &state, changes).unwrap();
        assert_eq!(admitted, 1);
        assert_eq!(filtered.len(), 2);
        assert_eq!(filtered[0]["name"], "src/__generated__/Owned.graphql.ts");
        assert!(
            !cfg.excludes
                .iter()
                .any(|pattern| pattern == "**/__generated__/**")
        );
    }

    #[test]
    fn source_removal_falls_back_but_missing_artifact_stays_incremental() {
        let cfg = Config::from_string_for_test(
            r#"{
          "sources": {"src": "app"},
          "projects": {"app": {"schema": "schema.graphql", "language": "typescript"}}
        }"#,
        )
        .unwrap();
        assert!(source_removal(
            &cfg,
            &[json!({"name": "src/Removed.ts", "exists": false})]
        ));
        assert!(!source_removal(
            &cfg,
            &[json!({"name": "src/__generated__/Owned.graphql.ts", "exists": false})]
        ));
        assert!(!source_removal(
            &cfg,
            &[json!({"name": "src/Changed.ts", "exists": true})]
        ));
    }

    #[tokio::test]
    async fn validation_persister_refuses_new_ids() {
        let error = ValidationPersister
            .persist_artifact(ArtifactForPersister {
                text: "query TestQuery { echo }".into(),
                relative_path: PathBuf::from("src/__generated__/TestQuery.graphql.ts"),
                override_schema: None,
            })
            .await
            .unwrap_err();
        assert!(error.to_string().contains("Artifact regeneration required"));
    }
}
