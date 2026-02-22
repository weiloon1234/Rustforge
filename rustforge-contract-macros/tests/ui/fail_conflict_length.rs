use rustforge_contract_macros::rustforge_contract;

#[rustforge_contract]
struct Input {
    #[rf(length(min = 1, equal = 3))]
    value: String,
}

fn main() {}
