use std::fs;
use std::path::PathBuf;

#[test]
fn template_workspace_dependencies_use_git_sources_for_framework_crates() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let template_manifest = manifest_dir.join("template").join("Cargo.toml");
    let content = fs::read_to_string(&template_manifest)
        .expect("failed to read scaffold/template/Cargo.toml");

    let crates = [
        "bootstrap",
        "core-config",
        "core-db",
        "core-datatable",
        "core-mailer",
        "core-i18n",
        "core-jobs",
        "core-notify",
        "core-realtime",
        "core-web",
        "db-gen",
    ];

    for crate_name in crates {
        let expected = format!(
            "{crate_name} = {{ git = \"https://github.com/weiloon1234/Rustforge.git\", branch = \"main\" }}"
        );
        assert!(
            content.contains(&expected),
            "template dependency for '{crate_name}' must use git source"
        );

        let local_path_pattern = format!("{crate_name} = {{ path = ");
        assert!(
            !content.contains(&local_path_pattern),
            "template dependency for '{crate_name}' must not use local path"
        );
    }
}
