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

fn run_cargo_check(output: &Path, package: &str) -> std::process::Output {
    let mut cmd = Command::new("cargo");
    cmd.arg("check").arg("-p").arg(package).current_dir(output);
    apply_local_rustforge_patches(&mut cmd);
    cmd.output().expect("failed to run cargo check")
}

fn apply_local_rustforge_patches(cmd: &mut Command) {
    const GIT_SOURCE: &str = "https://github.com/weiloon1234/Rustforge.git";
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let repo_root = manifest_dir
        .parent()
        .expect("scaffold crate should live under repo root");

    for crate_name in [
        "bootstrap",
        "core-config",
        "core-db",
        "core-datatable",
        "core-i18n",
        "core-jobs",
        "core-mailer",
        "core-notify",
        "core-realtime",
        "core-web",
        "db-gen",
        "rustforge-contract-macros",
        "rustforge-contract-meta",
    ] {
        let path = repo_root.join(crate_name);
        let patch = format!(
            "patch.\"{GIT_SOURCE}\".{crate_name}.path=\"{}\"",
            path.display()
        );
        cmd.arg("--config").arg(patch);
    }
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

    assert!(
        !out_dir.join("Cargo.lock").exists(),
        "scaffold output should not ship Cargo.lock"
    );

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
            "scripts/deploy.sh",
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
    }

    let agent_aliases = [
        "CLAUDE.md",
        "GEMINI.md",
        "frontend/CLAUDE.md",
        "frontend/GEMINI.md",
        "app/CLAUDE.md",
        "app/GEMINI.md",
    ];

    let deprecated_agent_artifacts = [
        "app/src/contracts/AGENTS.md",
        "app/src/contracts/CLAUDE.md",
        "app/src/contracts/GEMINI.md",
        "app/src/internal/AGENTS.md",
        "app/src/internal/CLAUDE.md",
        "app/src/internal/GEMINI.md",
        "app/src/seeds/AGENTS.md",
        "app/src/seeds/CLAUDE.md",
        "app/src/seeds/GEMINI.md",
        "app/src/validation/AGENTS.md",
        "app/src/validation/CLAUDE.md",
        "app/src/validation/GEMINI.md",
    ];

    #[cfg(unix)]
    {
        for rel in agent_aliases {
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
        for rel in agent_aliases {
            let path = out_dir.join(rel);
            assert!(path.is_file(), "expected copied file: {}", path.display());
            let linked = fs::read_to_string(&path)
                .unwrap_or_else(|e| panic!("failed to read {}: {e}", path.display()));
            let parent_agents = fs::read_to_string(path.parent().unwrap().join("AGENTS.md"))
                .expect("failed to read source AGENTS.md");
            assert_eq!(linked, parent_agents, "copied link file content mismatch");
        }
    }

    for rel in deprecated_agent_artifacts {
        assert!(
            !out_dir.join(rel).exists(),
            "deprecated AGENTS artifact should not exist in fresh scaffold output: {rel}"
        );
    }

    let check_generated = run_cargo_check(&out_dir, "generated");
    assert_ok(
        &check_generated,
        "fresh scaffold output should compile generated package",
    );

    let check_app = run_cargo_check(&out_dir, "app");
    assert_ok(
        &check_app,
        "fresh scaffold output should compile app package",
    );

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

    // Simulate stale artifacts from older scaffold versions to verify --force cleanup.
    for rel in deprecated_agent_artifacts {
        let path = out_dir.join(rel);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).unwrap_or_else(|e| {
                panic!(
                    "failed to create stale artifact parent {}: {e}",
                    parent.display()
                )
            });
        }
        fs::write(&path, b"stale").unwrap_or_else(|e| {
            panic!(
                "failed to create stale artifact file {}: {e}",
                path.display()
            )
        });
    }

    let second_force = run_scaffold(&out_dir, true);
    assert_ok(
        &second_force,
        "scaffold --force rerun in non-empty dir should succeed",
    );

    for rel in deprecated_agent_artifacts {
        assert!(
            !out_dir.join(rel).exists(),
            "deprecated AGENTS artifact should be removed by --force rerun: {rel}"
        );
    }

    let _ = fs::remove_dir_all(&out_dir);
}
