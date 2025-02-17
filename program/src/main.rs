#![no_main]
sp1_zkvm::entrypoint!(main);

use svm_sp1_test_lib::transfer_sols;

pub fn main() {
    // let alice_starting_lamports = 1 * LAMPORTS_PER_SOL;
    // let bob_starting_lamports = 1 * LAMPORTS_PER_SOL;
    // let amount = 666;

    let alice_starting_lamports = sp1_zkvm::io::read::<u64>();
    let bob_starting_lamports = sp1_zkvm::io::read::<u64>();
    let amount = sp1_zkvm::io::read::<u64>();

    let result = transfer_sols(alice_starting_lamports, bob_starting_lamports, amount);

    // match result {
    //     Ok((alice, bob)) => {
    //         println!("Alice: {}", alice);
    //         println!("Bob: {}", bob);
    //     }
    //     Err(err) => {
    //         println!("Error: {}", err);
    //     }
    // }

    let result = result.unwrap();

    sp1_zkvm::io::commit(&result.0);
    sp1_zkvm::io::commit(&result.1);
}
