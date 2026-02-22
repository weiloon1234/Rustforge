use rustforge_contract_macros::rustforge_contract;
use serde::Deserialize;
use validator::Validate;

#[rustforge_contract]
#[derive(Debug, Deserialize, Validate, schemars::JsonSchema)]
struct ChildInput {
    #[rf(length(min = 1, max = 16))]
    name: String,
}

#[rustforge_contract]
#[derive(Debug, Deserialize, Validate, schemars::JsonSchema)]
struct ParentInput {
    #[rf(nested)]
    child: ChildInput,

    #[rf(length(min = 8, max = 64))]
    password: String,

    #[rf(length(min = 8, max = 64))]
    #[rf(must_match(other = "password"))]
    password_confirmation: String,

}

fn main() {}
