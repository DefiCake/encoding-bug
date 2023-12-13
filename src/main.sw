contract;

use std::math::*;

abi MyContract {
    fn test_function(dividend: u256, divisor: u256) -> u256;
}

const TEN: u256 = 0x0a_u256;

impl MyContract for Contract {
    fn test_function(dividend: u256, divisor: u256) -> u256 {
        // This shouldnt affect the end result in any way... but
        let pow_op = TEN.pow(9u32); // Comment this line and the test passes


        dividend % divisor
    }
}