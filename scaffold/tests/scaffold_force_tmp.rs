use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

fn unique_output_dir() -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time should be after UNIX_EPOCH")
        .as_nanos();
    std::env::temp_dir().join(format!(
        "rustforge-scaffold-force-tmp-{}-{nanos}",
        std::process::id()
    ))
}

fn run_scaffold(output: &Path) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_scaffold"))
        .arg("--output")
        .arg(output)
        .arg("--force")
        .arg("--project-name")
        .arg("testproject")
        .arg("--bucket-name")
        .arg("testbucket")
        .output()
        .expect("failed to run scaffold binary")
}

fn assert_ok(output: &std::process::Output, context: &str) {
    assert!(
        output.status.success(),
        "{context}\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn scaffold_force_into_tmp_uses_model_sources_layout() {
    let out_dir = unique_output_dir();

    if out_dir.exists() {
        let _ = fs::remove_dir_all(&out_dir);
    }

    let first = run_scaffold(&out_dir);
    assert_ok(
        &first,
        "initial scaffold --force run into /tmp should succeed",
    );

    for rel in [
        "Cargo.toml",
        "app/models/admin.rs",
        "app/models/user.rs",
        "generated/build.rs",
        "generated/src/models/mod.rs",
    ] {
        assert!(
            out_dir.join(rel).is_file(),
            "expected generated file missing: {rel}"
        );
    }

    for rel in [
        "app/schemas",
        "app/src/internal/extensions",
        "generated/framework-schemas",
        "generated/src/extensions.rs",
    ] {
        assert!(
            !out_dir.join(rel).exists(),
            "deprecated scaffold artifact should not exist: {rel}"
        );
    }

    let second = run_scaffold(&out_dir);
    assert_ok(
        &second,
        "second scaffold --force run into same /tmp dir should succeed",
    );

    let _ = fs::remove_dir_all(&out_dir);
}
