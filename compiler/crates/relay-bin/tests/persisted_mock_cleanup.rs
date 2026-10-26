/*
 * Copyright (c) Meta Platforms, Inc. and affiliates.
 *
 * This source code is licensed under the MIT license found in the
 * LICENSE file in the root directory of this source tree.
 */

use std::fs;
use std::path::Path;
use std::process::Command;
use std::process::Output;

use graphql_test_helpers::TestDir;
use serde_json::json;

fn write(root: &Path, name: &str, content: &str) {
    let path = root.join(name);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, content).unwrap();
}

fn compile(root: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_relay"))
        .arg(root.join("relay.config.json"))
        .args(args)
        .env("FORCE_NO_WATCHMAN", "1")
        .current_dir(root)
        .output()
        .unwrap()
}

fn output_text(output: &Output) -> String {
    format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    )
}

fn assert_success(output: Output) {
    assert!(output.status.success(), "{}", output_text(&output));
}

fn check_mock_cleanup(delete_source: bool, keep_other_query: bool, mocks: &str) {
    let test_dir = TestDir::new();
    let root = test_dir.path();
    write(root, "schema.graphql", "type Query { greeting: String }\n");
    write(root, "persisted.json", "{}");
    write(
        root,
        "relay.config.json",
        &json!({
            "root": root,
            "sources": {"src": "jira", "disabled": "disabled"},
            // AFM excludes ordinary Relay artifacts; mocks must still be discovered.
            "excludes": ["**/__generated__/**"],
            "projects": {
                "jira": {
                    "schema": "schema.graphql",
                    "language": "typescript",
                    "extraArtifactsOutput": mocks,
                    "persist": {"file": root.join("persisted.json")}
                },
                "disabled": {
                    "schema": "schema.graphql",
                    "language": "typescript",
                    "extraArtifactsOutput": "disabled-mocks"
                }
            }
        })
        .to_string(),
    );
    fs::create_dir_all(root.join("disabled")).unwrap();
    write(root, "disabled-mocks/OtherQuery.json", "{}");
    write(root, "src/unrelated.json", "{}");
    // Keep the source set present even when its last GraphQL document is removed.
    write(root, "src/Unrelated.ts", "export const unrelated = true;");
    write(
        root,
        "src/Removed.ts",
        "const query = graphql`query RemovedQuery { greeting }`;",
    );
    if keep_other_query {
        write(
            root,
            "src/Retained.ts",
            "const query = graphql`query RetainedQuery { greeting }`;",
        );
    }
    assert_success(compile(root, &["--project", "jira"]));
    let mock = root.join(mocks).join("RemovedQuery.json");
    assert!(mock.is_file());
    let original_mock = fs::read(&mock).unwrap();
    // An unchanged compilation is valid, including its persisted mocks.
    assert_success(compile(root, &["--project", "jira", "--validate"]));

    if delete_source {
        fs::remove_file(root.join("src/Removed.ts")).unwrap();
    } else {
        write(root, "src/Removed.ts", "export const queryRemoved = true;");
    }

    let validation = compile(root, &["--project", "jira", "--validate"]);
    assert!(!validation.status.success(), "Stale mock was not detected");
    assert!(
        output_text(&validation).contains("RemovedQuery.json"),
        "{}",
        output_text(&validation)
    );
    assert_eq!(fs::read(&mock).unwrap(), original_mock);

    assert_success(compile(root, &["--project", "jira"]));
    assert!(!mock.exists(), "Removed query's mock was left behind");
    if keep_other_query {
        assert!(root.join(mocks).join("RetainedQuery.json").is_file());
    }
    assert!(root.join("src/unrelated.json").is_file());
    assert!(root.join("disabled-mocks/OtherQuery.json").is_file());
    assert_success(compile(root, &["--project", "jira", "--validate"]));
}

#[test]
fn removes_mock_when_graphql_tag_is_removed() {
    check_mock_cleanup(false, true, "src/mocks");
}

#[test]
fn removes_mock_when_source_file_is_deleted() {
    check_mock_cleanup(true, true, "mocks");
}

#[test]
fn removes_mock_when_last_graphql_tag_is_removed() {
    check_mock_cleanup(false, false, "mocks");
}

#[test]
fn removes_mock_when_last_query_source_file_is_deleted() {
    check_mock_cleanup(true, false, "src/mocks");
}
