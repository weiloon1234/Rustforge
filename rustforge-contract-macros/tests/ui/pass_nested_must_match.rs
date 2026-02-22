use rustforge_contract_macros::rustforge_contract;
use validator::Validate;

#[rustforge_contract]
struct ChildInput {
    #[rf(length(min = 1, max = 16))]
    name: String,
}

#[rustforge_contract]
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
