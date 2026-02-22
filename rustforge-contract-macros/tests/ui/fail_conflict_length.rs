use rustforge_contract_macros::rustforge_contract;
use serde::Deserialize;
use validator::Validate;

#[rustforge_contract]
#[derive(Debug, Deserialize, Validate, schemars::JsonSchema)]
struct Input {
    #[rf(length(min = 1, equal = 3))]
    value: String,
}

fn main() {}

