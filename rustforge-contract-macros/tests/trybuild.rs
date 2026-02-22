#[test]
fn rustforge_contract_trybuild() {
    let t = trybuild::TestCases::new();
    t.pass("tests/ui/pass_basic.rs");
    t.pass("tests/ui/pass_nested_must_match.rs");
    t.compile_fail("tests/ui/fail_unknown_rule.rs");
    t.compile_fail("tests/ui/fail_conflict_length.rs");
}
