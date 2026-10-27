/*
 * Copyright (c) Atlassian and its affiliates.
 *
 * This source code is licensed under the MIT license found in the
 * LICENSE file in the root directory of this source tree.
 */

use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::process::Output;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;

use serde_json::json;

static NEXT_FIXTURE: AtomicUsize = AtomicUsize::new(0);

struct Fixture {
    root: PathBuf,
}

impl Fixture {
    fn new() -> Self {
        let root = std::env::temp_dir().join(format!(
            "relay-cli-state-{}-{}",
            std::process::id(),
            NEXT_FIXTURE.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&root).unwrap();
        fs::create_dir(root.join("src")).unwrap();
        fs::create_dir(root.join("runner")).unwrap();
        fs::write(
            root.join("schema.graphql"),
            "type Query { viewer: User } type User { id: ID!, name: String }",
        )
        .unwrap();
        fs::write(
            root.join("relay.config.json"),
            json!({
                "root": ".",
                "sources": {"src": "app"},
                "projects": {"app": {"schema": "schema.graphql", "language": "typescript"}},
                "noSourceControl": true
            })
            .to_string(),
        )
        .unwrap();
        fs::write(
            root.join("src/App.ts"),
            "const fragment = graphql`fragment App_user on User { id }`;",
        )
        .unwrap();
        fs::write(root.join("changes.json"), "[]").unwrap();
        Self { root }
    }

    fn run(&self, args: &[&str]) -> Output {
        Command::new(env!("CARGO_BIN_EXE_relay"))
            // Deliberately different from config.root, including for omitted exists.
            .current_dir(self.root.join("runner"))
            .env("FORCE_NO_WATCHMAN", "1")
            .env_remove("RELAY_COMPILER_IGNORE_SAVED_STATE_VERSION")
            .arg(self.root.join("relay.config.json"))
            .args(args)
            .output()
            .unwrap()
    }

    fn seed(&self) {
        success(self.run(&["--state-file", "../state.bin"]));
        assert!(self.root.join("state.bin").is_file());
        assert!(
            self.root
                .join("src/__generated__/App_user.graphql.ts")
                .is_file()
        );
    }

    fn changes(&self, value: serde_json::Value) {
        fs::write(self.root.join("changes.json"), value.to_string()).unwrap();
    }

    fn incremental(&self, validate: bool) -> Output {
        let mut args = vec![
            "--state-file",
            "../state.bin",
            "--changed-files",
            "../changes.json",
        ];
        if validate {
            args.push("--validate");
        }
        self.run(&args)
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

fn success(output: Output) {
    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

fn failure(output: Output) -> String {
    assert!(!output.status.success());
    format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    )
}

#[test]
fn full_and_incremental_validation_save_reusable_state() {
    let fixture = Fixture::new();
    fixture.seed();
    success(fixture.run(&["--validate", "--state-file", "../validated.bin"]));
    success(fixture.incremental(true));
    success(fixture.incremental(true));
    // Without a delta, even an invalid existing checkpoint is replaced by a
    // full compilation. It must not be loaded or treated as an empty delta.
    fs::write(fixture.root.join("state.bin"), "old incompatible state").unwrap();
    success(fixture.run(&["--validate", "--state-file", "../state.bin"]));
    success(fixture.incremental(true));
}

#[test]
fn stale_sources_fail_without_replacing_output_state_then_regenerate_incrementally() {
    let fixture = Fixture::new();
    fixture.seed();
    fs::write(
        fixture.root.join("src/App.ts"),
        "const fragment = graphql`fragment App_user on User { id name }`;",
    )
    .unwrap();
    fixture.changes(json!([{"name": "src/App.ts", "exists": true}]));
    let original = fs::read(fixture.root.join("state.bin")).unwrap();
    failure(fixture.incremental(true));
    assert_eq!(fs::read(fixture.root.join("state.bin")).unwrap(), original);
    // Full validation failure also preserves the previous checkpoint.
    failure(fixture.run(&["--validate", "--state-file", "../state.bin"]));
    assert_eq!(fs::read(fixture.root.join("state.bin")).unwrap(), original);
    success(fixture.incremental(false));
    fixture.changes(json!([]));
    success(fixture.incremental(true));
    success(fixture.run(&["--validate"]));
}

#[test]
fn artifact_only_deletion_or_edit_cannot_pass_validation() {
    for delete in [false, true] {
        let fixture = Fixture::new();
        fixture.seed();
        let artifact = fixture.root.join("src/__generated__/App_user.graphql.ts");
        if delete {
            fs::remove_file(&artifact).unwrap();
        } else {
            fs::write(&artifact, "invalid artifact").unwrap();
        }
        fixture
            .changes(json!([{"name": "src/__generated__/App_user.graphql.ts", "exists": !delete}]));
        let original = fs::read(fixture.root.join("state.bin")).unwrap();
        failure(fixture.incremental(true));
        assert_eq!(fs::read(fixture.root.join("state.bin")).unwrap(), original);
        success(fixture.incremental(false));
        success(fixture.run(&["--validate"]));
    }
}

#[test]
fn omitted_exists_is_resolved_against_config_root() {
    let fixture = Fixture::new();
    fixture.seed();
    fs::write(
        fixture.root.join("src/Other.ts"),
        "const fragment = graphql`fragment Other_user on User { name }`;",
    )
    .unwrap();
    fixture.changes(json!([{"name": "src/Other.ts"}]));
    failure(fixture.incremental(true));
    success(fixture.incremental(false));
    assert!(
        fixture
            .root
            .join("src/__generated__/Other_user.graphql.ts")
            .is_file()
    );
    success(fixture.run(&["--validate"]));
}

#[test]
fn invalid_state_or_delta_fails_without_watchman_or_output_state() {
    for case in [
        "missing-state",
        "corrupt-state",
        "missing-delta",
        "malformed-delta",
        "version-mismatch",
        "traversal",
        "absolute",
    ] {
        let fixture = Fixture::new();
        fixture.seed();
        match case {
            "missing-state" => fs::remove_file(fixture.root.join("state.bin")).unwrap(),
            "corrupt-state" => fs::write(fixture.root.join("state.bin"), "not state").unwrap(),
            "missing-delta" => fs::remove_file(fixture.root.join("changes.json")).unwrap(),
            "malformed-delta" => fs::write(fixture.root.join("changes.json"), "not json").unwrap(),
            "version-mismatch" => {
                let path = fixture.root.join("relay.config.json");
                let mut config: serde_json::Value =
                    serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
                config["codegenCommand"] = json!("different-codegen-command");
                fs::write(path, config.to_string()).unwrap();
            }
            "traversal" => fixture.changes(json!([{"name": "../outside.ts", "exists": true}])),
            "absolute" => fixture.changes(json!([{"name": "/outside.ts", "exists": true}])),
            _ => unreachable!(),
        }
        let original = fs::read(fixture.root.join("state.bin")).ok();
        let diagnostic = failure(fixture.incremental(true));
        assert!(
            !diagnostic.to_lowercase().contains("watchman"),
            "{case}: {diagnostic}"
        );
        assert_eq!(
            fs::read(fixture.root.join("state.bin")).ok(),
            original,
            "{case}"
        );
    }
}

#[test]
fn checkpoint_write_failures_are_reported_and_temporary_files_are_cleaned_up() {
    let fixture = Fixture::new();
    fixture.seed();
    fs::create_dir(fixture.root.join("destination")).unwrap();
    fs::write(fixture.root.join("destination/keep"), "unchanged").unwrap();
    // Renaming a completed checkpoint over a directory must fail, not remove it.
    failure(fixture.run(&["--validate", "--state-file", "../destination"]));
    assert_eq!(
        fs::read(fixture.root.join("destination/keep")).unwrap(),
        b"unchanged"
    );
    failure(fixture.run(&["--validate", "--state-file", "../missing/state.bin"]));
    assert!(!fixture.root.join("missing").exists());
    assert!(fs::read_dir(&fixture.root).unwrap().all(|entry| {
        !entry
            .unwrap()
            .file_name()
            .to_string_lossy()
            .starts_with(".relay-state-")
    }));
}

#[cfg(target_os = "linux")]
#[test]
fn checkpoint_serialization_reports_buffered_write_failures() {
    let fixture = Fixture::new();
    fixture.seed();
    let state = relay_compiler::compiler_state::CompilerState::deserialize_from_file(
        &fixture.root.join("state.bin"),
    )
    .unwrap();
    assert!(
        state
            .serialize_to_file(&PathBuf::from("/dev/full"))
            .is_err()
    );
}

#[test]
fn cli_rejects_unpaired_flags_and_incompatible_modes() {
    let fixture = Fixture::new();
    for args in [
        vec!["--load-saved-state", "state.bin"],
        vec!["--save-state", "state.bin"],
        vec!["--changed-files", "changes.json"],
        vec!["--state-file", "state.bin", "--watch"],
        vec!["--state-file", "state.bin", "--project", "app"],
        vec!["--state-file", "state.bin", "--repersist"],
        vec![
            "--state-file",
            "state.bin",
            "--changed-files",
            "changes.json",
            "--watch",
        ],
    ] {
        let output = fixture.run(&args);
        assert_eq!(
            output.status.code(),
            Some(2),
            "{args:?}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
}
