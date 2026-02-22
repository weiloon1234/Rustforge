use rustforge_contract_macros::rustforge_contract;

#[rustforge_contract]
struct Input {
    #[rf(not_a_real_rule)]
    value: String,
}

fn main() {}
