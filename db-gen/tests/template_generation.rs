use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use db_gen::{
    config, generate_auth, generate_datatable_skeletons, generate_enums, generate_localized,
    generate_models, generate_permissions, load_permissions, schema,
};

fn temp_dir(prefix: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock should be after unix epoch")
        .as_nanos();
    let dir = std::env::temp_dir().join(format!(
        "rs_db_gen_fixture_{prefix}_{}_{}",
        std::process::id(),
        nanos
    ));
    fs::create_dir_all(&dir).expect("failed to create temp dir");
    dir
}

fn fixture_root(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join(name)
}

fn assert_fixture_eq(got_path: &Path, expected_path: &Path) {
    let got = fs::read_to_string(got_path).unwrap_or_else(|err| {
        panic!(
            "failed to read generated fixture '{}': {}",
            got_path.display(),
            err
        )
    });
    if std::env::var_os("UPDATE_DB_GEN_FIXTURES").is_some() {
        if let Some(parent) = expected_path.parent() {
            fs::create_dir_all(parent).expect("failed to create fixture parent dir");
        }
        fs::write(expected_path, &got).unwrap_or_else(|err| {
            panic!(
                "failed to update fixture '{}': {}",
                expected_path.display(),
                err
            )
        });
    }
    let expected = fs::read_to_string(expected_path).unwrap_or_else(|err| {
        panic!(
            "failed to read expected fixture '{}': {}",
            expected_path.display(),
            err
        )
    });
    assert_eq!(
        got,
        expected,
        "fixture mismatch for {}",
        expected_path.display()
    );
}

#[test]
fn generator_outputs_match_checked_in_fixtures() {
    let fixture = fixture_root("full_stack");
    let inputs = fixture.join("inputs");
    let expected = fixture.join("expected");
    let out = temp_dir("full_stack");

    let permissions = load_permissions(
        inputs
            .join("permissions.toml")
            .to_str()
            .expect("permission fixture path should be valid utf-8"),
    )
    .expect("failed to load permission fixture");
    generate_permissions(&permissions, &out.join("permissions.rs"))
        .expect("permission generation should succeed");

    let (cfgs, _) = config::load(
        inputs
            .join("configs.toml")
            .to_str()
            .expect("config fixture path should be valid utf-8"),
    )
    .expect("failed to load config fixture");
    let parsed_schema = schema::load(
        inputs
            .join("schemas")
            .to_str()
            .expect("schema fixture path should be valid utf-8"),
    )
    .expect("failed to load schema fixture");

    let auth_out = out.join("auth");
    let datatable_out = out.join("datatables");
    let model_out = out.join("models");
    fs::create_dir_all(&auth_out).expect("failed to create auth out dir");
    fs::create_dir_all(&datatable_out).expect("failed to create datatable out dir");
    fs::create_dir_all(&model_out).expect("failed to create model out dir");

    generate_auth(&cfgs, &parsed_schema, &auth_out).expect("auth generation should succeed");
    generate_datatable_skeletons(&parsed_schema, &datatable_out)
        .expect("datatable generation should succeed");
    generate_enums(&parsed_schema, &model_out).expect("enum generation should succeed");
    generate_models(&parsed_schema, &cfgs, &model_out).expect("model generation should succeed");
    generate_localized(&cfgs.languages, &cfgs, &parsed_schema, &model_out)
        .expect("localized generation should succeed");

    assert_fixture_eq(
        &out.join("permissions.rs"),
        &expected.join("permissions.rs"),
    );
    assert_fixture_eq(
        &auth_out.join("admin_guard.rs"),
        &expected.join("auth/admin_guard.rs"),
    );
    assert_fixture_eq(&auth_out.join("mod.rs"), &expected.join("auth/mod.rs"));
    assert_fixture_eq(
        &datatable_out.join("mod.generated.rs"),
        &expected.join("datatables/mod.generated.rs"),
    );
    assert_fixture_eq(
        &datatable_out.join("article.rs"),
        &expected.join("datatables/article.rs"),
    );
    assert_fixture_eq(
        &model_out.join("enums.rs"),
        &expected.join("models/enums.rs"),
    );
    assert_fixture_eq(
        &model_out.join("localized.rs"),
        &expected.join("models/localized.rs"),
    );
    assert_fixture_eq(
        &model_out.join("article.rs"),
        &expected.join("models/article.rs"),
    );
    assert_fixture_eq(&model_out.join("mod.rs"), &expected.join("models/mod.rs"));
    assert_fixture_eq(
        &model_out.join("common.rs"),
        &expected.join("models/common.rs"),
    );

    fs::remove_dir_all(out).expect("failed to remove temp dir");
}
