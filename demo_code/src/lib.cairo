#[starknet::interface]
trait MyTrait<T> {
    fn my_method(self: @T) -> i32;
}

#[custom::contract]
mod contract {
    #[storage]
    struct Storage {}

    impl MyTraitImpl of super::MyTrait<ContractState> {
        fn my_method(self: @ContractState) -> i32 {
            // Diagnostic pointer is correct as this is not re-written.
            // return 1_u256
            42
        }
    }
}
