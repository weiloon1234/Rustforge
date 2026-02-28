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
        "rustforge-scaffold-it-{}-{nanos}",
        std::process::id()
    ))
}

fn run_scaffold(output: &Path, force: bool) -> std::process::Output {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_scaffold"));
    cmd.arg("--output").arg(output);
    if force {
        cmd.arg("--force");
    }
    cmd.output().expect("failed to run scaffold binary")
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
fn scaffold_smoke_generation_and_force_behaviour() {
    let out_dir = unique_output_dir();

    if out_dir.exists() {
        let _ = fs::remove_dir_all(&out_dir);
    }

    let first = run_scaffold(&out_dir, true);
    assert_ok(&first, "initial scaffold --force run should succeed");

    for rel in [
        "Cargo.toml",
        "app/src/lib.rs",
        "frontend/src/admin/App.tsx",
        "generated/build.rs",
        "frontend/src/shared/useAutoForm.tsx",
    ] {
        assert!(
            out_dir.join(rel).is_file(),
            "expected generated file missing: {rel}"
        );
    }

    let env_example =
        fs::read_to_string(out_dir.join(".env.example")).expect("failed to read .env.example");
    assert!(
        env_example.contains("APP_KEY=base64:"),
        ".env.example should contain rendered APP_KEY"
    );
    assert!(
        !env_example.contains("{{APP_KEY}}"),
        ".env.example should not contain APP_KEY placeholder"
    );

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;

        for rel in [
            "console",
            "bin/api-server",
            "bin/websocket-server",
            "bin/worker",
            "bin/console",
            "scripts/install-ubuntu.sh",
            "scripts/update.sh",
        ] {
            let path = out_dir.join(rel);
            let mode = fs::metadata(&path)
                .unwrap_or_else(|e| panic!("failed to stat {}: {e}", path.display()))
                .permissions()
                .mode();
            assert!(
                mode & 0o111 != 0,
                "expected executable bit on {}",
                path.display()
            );
        }

        for rel in [
            "CLAUDE.md",
            "GEMINI.md",
            "frontend/CLAUDE.md",
            "frontend/GEMINI.md",
            "app/src/contracts/CLAUDE.md",
            "app/src/contracts/GEMINI.md",
        ] {
            let path = out_dir.join(rel);
            let file_type = fs::symlink_metadata(&path)
                .unwrap_or_else(|e| panic!("failed to stat {}: {e}", path.display()))
                .file_type();
            assert!(
                file_type.is_symlink(),
                "expected symlink: {}",
                path.display()
            );

            let target = fs::read_link(&path)
                .unwrap_or_else(|e| panic!("failed to read symlink {}: {e}", path.display()));
            assert_eq!(
                target,
                PathBuf::from("AGENTS.md"),
                "unexpected symlink target for {}",
                path.display()
            );
        }
    }

    #[cfg(not(unix))]
    {
        for rel in [
            "CLAUDE.md",
            "GEMINI.md",
            "frontend/CLAUDE.md",
            "frontend/GEMINI.md",
            "app/src/contracts/CLAUDE.md",
            "app/src/contracts/GEMINI.md",
        ] {
            let path = out_dir.join(rel);
            assert!(path.is_file(), "expected copied file: {}", path.display());
            let linked = fs::read_to_string(&path)
                .unwrap_or_else(|e| panic!("failed to read {}: {e}", path.display()));
            let parent_agents = fs::read_to_string(path.parent().unwrap().join("AGENTS.md"))
                .expect("failed to read source AGENTS.md");
            assert_eq!(linked, parent_agents, "copied link file content mismatch");
        }
    }

    let no_force = run_scaffold(&out_dir, false);
    assert!(
        !no_force.status.success(),
        "scaffold should fail without --force in non-empty dir"
    );
    let combined = format!(
        "{}\n{}",
        String::from_utf8_lossy(&no_force.stdout),
        String::from_utf8_lossy(&no_force.stderr)
    );
    assert!(
        combined.contains("Refusing to scaffold into non-empty directory"),
        "expected non-empty output error message"
    );

    let second_force = run_scaffold(&out_dir, true);
    assert_ok(
        &second_force,
        "scaffold --force rerun in non-empty dir should succeed",
    );

    let _ = fs::remove_dir_all(&out_dir);
}
