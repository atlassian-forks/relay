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
        self.run_binary(env!("CARGO_BIN_EXE_relay-compiler-ci"), args)
    }

    fn run_stock(&self, args: &[&str]) -> Output {
        self.run_binary(env!("CARGO_BIN_EXE_relay"), args)
    }

    fn run_binary(&self, binary: &str, args: &[&str]) -> Output {
        Command::new(binary)
            // Deliberately different from config.root, including for omitted exists.
            .current_dir(self.root.join("runner"))
            .env("FORCE_NO_WATCHMAN", "1")
            // No host Watchman executable: external-state fallback must fail
            // deterministically instead of connecting to a developer's daemon.
            .env("PATH", self.root.join("runner"))
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
fn fragment_changes_reach_transitive_operations_without_dirty_operation_inputs() {
    let fixture = Fixture::new();
    fs::write(
        fixture.root.join("src/Middle.ts"),
        "const node = graphql`fragment Middle_user on User { ...App_user }`;",
    )
    .unwrap();
    fs::write(
        fixture.root.join("src/Viewer.ts"),
        "const node = graphql`query ViewerQuery { viewer { ...Middle_user } }`;",
    )
    .unwrap();
    fixture.seed();
    let operation = fixture
        .root
        .join("src/__generated__/ViewerQuery.graphql.ts");
    let middle = fixture
        .root
        .join("src/__generated__/Middle_user.graphql.ts");
    let original_operation = fs::read(&operation).unwrap();
    let original_middle = fs::read(&middle).unwrap();
    fs::write(
        fixture.root.join("src/App.ts"),
        "const node = graphql`fragment App_user on User { id name }`;",
    )
    .unwrap();
    success(fixture.run_stock(&[]));
    let correct_operation = fs::read(&operation).unwrap();
    let correct_middle = fs::read(&middle).unwrap();
    assert_ne!(original_operation, correct_operation);
    fs::write(&operation, original_operation).unwrap();
    fs::write(&middle, original_middle).unwrap();
    fixture.changes(json!([
        {"name": "src/App.ts", "exists": true},
        {"name": "src/__generated__/App_user.graphql.ts", "exists": true}
    ]));
    let original_state = fs::read(fixture.root.join("state.bin")).unwrap();
    let diagnostic = failure(fixture.incremental(true));
    assert!(
        diagnostic.contains("ViewerQuery.graphql.ts"),
        "{diagnostic}"
    );
    assert_eq!(
        fs::read(fixture.root.join("state.bin")).unwrap(),
        original_state
    );
    fs::write(&operation, correct_operation).unwrap();
    fs::write(&middle, correct_middle).unwrap();
    fixture.changes(json!([
        {"name": "src/App.ts", "exists": true},
        {"name": "src/__generated__/App_user.graphql.ts", "exists": true},
        {"name": "src/__generated__/Middle_user.graphql.ts", "exists": true},
        {"name": "src/__generated__/ViewerQuery.graphql.ts", "exists": true}
    ]));
    success(fixture.incremental(true));
    fixture.changes(json!([]));
    success(fixture.incremental(true));
}

#[test]
fn artifacts_from_other_tools_are_not_rewritten() {
    let fixture = Fixture::new();
    fixture.seed();
    let other = fixture.root.join("src/__generated__/other.js");
    fs::write(&other, "other tool output").unwrap();
    fixture.changes(json!([{"name": "src/__generated__/other.js", "exists": true}]));
    success(fixture.incremental(true));
    assert_eq!(fs::read(&other).unwrap(), b"other tool output");
    fs::remove_file(fixture.root.join("src/__generated__/App_user.graphql.ts")).unwrap();
    fixture.changes(json!([{"name": "src/__generated__/App_user.graphql.ts", "exists": false}]));
    failure(fixture.incremental(true));
}

#[test]
fn caller_requires_explicit_exists_without_changing_native_reader() {
    let fixture = Fixture::new();
    fixture.seed();
    fs::write(
        fixture.root.join("src/Other.ts"),
        "const fragment = graphql`fragment Other_user on User { name }`;",
    )
    .unwrap();
    fixture.changes(json!([{"name": "src/Other.ts"}]));
    let error = failure(fixture.incremental(true));
    assert!(error.contains("missing field `exists`"), "{error}");
    fixture.changes(json!([{"name": "src/Other.ts", "exists": true}]));
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
fn invalid_external_inputs_retain_upstream_watchman_fallback() {
    for case in ["missing-state", "corrupt-state", "version-mismatch"] {
        let fixture = Fixture::new();
        fixture.seed();
        match case {
            "missing-state" => fs::remove_file(fixture.root.join("state.bin")).unwrap(),
            "corrupt-state" => fs::write(fixture.root.join("state.bin"), "not state").unwrap(),
            "version-mismatch" => {
                let path = fixture.root.join("relay.config.json");
                let mut config: serde_json::Value =
                    serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
                config["codegenCommand"] = json!("different-codegen-command");
                fs::write(path, config.to_string()).unwrap();
            }
            _ => unreachable!(),
        }
        let original = fs::read(fixture.root.join("state.bin")).ok();
        let diagnostic = failure(fixture.incremental(true));
        assert!(
            diagnostic.to_lowercase().contains("watchman"),
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
fn invalid_delta_is_rejected_by_the_caller_without_watchman() {
    let fixture = Fixture::new();
    fixture.seed();
    let original = fs::read(fixture.root.join("state.bin")).unwrap();
    for name in [
        "",
        "../outside.ts",
        "/outside.ts",
        "C:/outside.ts",
        "src\\Other.ts",
    ] {
        fixture.changes(json!([{"name": name, "exists": true}]));
        let error = failure(fixture.incremental(true));
        assert!(error.contains("Invalid root-relative"), "{error}");
        assert!(!error.to_lowercase().contains("watchman"), "{error}");
    }
    fs::write(fixture.root.join("changes.json"), "not json").unwrap();
    failure(fixture.incremental(true));
    fs::remove_file(fixture.root.join("changes.json")).unwrap();
    failure(fixture.incremental(true));
    assert_eq!(fs::read(fixture.root.join("state.bin")).unwrap(), original);
}

#[test]
fn ordinary_cli_does_not_expose_checkpoint_flags() {
    let fixture = Fixture::new();
    success(fixture.run_stock(&[]));
    success(fixture.run_stock(&["--validate"]));
    assert_eq!(
        fixture
            .run_stock(&["--state-file", "../state.bin"])
            .status
            .code(),
        Some(2)
    );
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
