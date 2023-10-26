contract;

abi MyContract {
    fn test_function(to: u64) -> u64;
}

impl MyContract for Contract {
    fn test_function(to: u64) -> u64 {
        log(to);
        log(666);

        to
    }
}
