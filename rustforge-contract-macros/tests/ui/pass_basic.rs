use rustforge_contract_macros::rustforge_contract;

#[rustforge_contract]
struct Input {
    #[rf(length(min = 3, max = 32))]
    #[rf(email)]
    email: String,
}

fn main() {}
