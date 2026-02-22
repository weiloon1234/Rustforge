use rustforge_contract_macros::rustforge_contract;
use serde::Deserialize;
use validator::Validate;

#[rustforge_contract]
#[derive(Debug, Deserialize, Validate, schemars::JsonSchema)]
struct Input {
    #[rf(rule = "not_a_real_rule")]
    value: String,
}

fn main() {}

