contract;

use std::inputs::input_message_sender;

abi MyContract {
    #[payable]
    #[storage(read, write)]
    fn test_function(to: u64) -> u64;
}

abi MessageReceiver {
    #[payable]
    #[storage(read, write)]
    fn process_message(msg_idx: u64);
}


impl MyContract for Contract {
    #[payable]
    #[storage(read, write)]
    fn test_function(msg_idx: u64) -> u64 {
        let input_sender = input_message_sender(msg_idx);
        log(input_sender);

        0u64
    }
}

impl MessageReceiver for Contract {
    #[payable]
    #[storage(read, write)]
    fn process_message(msg_idx: u64) {
        let input_sender = input_message_sender(msg_idx);
        log(input_sender);
    }
}

// predicate;

// fn main() -> bool {
//     true
// }