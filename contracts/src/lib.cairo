#[starknet::interface]
trait MyTrait<T> {
    fn my_method(self: @T) -> i32;
}

#[custom::contract]
mod contract {
    #[storage]
    struct Storage {}

    impl notbad of super::MyTrait<ContractState> {
        // Self will be automatically added by the plugin.
        // Or use `r: R` to inject ref self instead.
        fn my_method() -> i32 {
            42
        }
    }
}
