use rustforge_contract_macros::rustforge_contract;
use serde::Deserialize;
use validator::Validate;

#[rustforge_contract]
#[derive(Debug, Deserialize, Validate, schemars::JsonSchema)]
struct Input {
    #[rf(length(min = 3, max = 32))]
    #[rf(email)]
    email: String,
}

fn main() {}

