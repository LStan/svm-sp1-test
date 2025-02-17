use solana_sdk::native_token::LAMPORTS_PER_SOL;
use sp1_sdk::{include_elf, ProverClient, SP1Stdin};

pub const SVM_SP1_TEST_ELF: &[u8] = include_elf!("svm-sp1-test-program");

fn main() {
    let alice_starting_lamports = 1 * LAMPORTS_PER_SOL;
    let bob_starting_lamports = 1 * LAMPORTS_PER_SOL;
    let amount = 666u64;

    let mut stdin = SP1Stdin::new();
    stdin.write(&alice_starting_lamports);
    stdin.write(&bob_starting_lamports);
    stdin.write(&amount);

    let client = ProverClient::from_env();

    let (mut output, report) = client.execute(SVM_SP1_TEST_ELF, &stdin).run().unwrap();

    let alice: u64 = output.read();
    let bob: u64 = output.read();
    println!("alice: {}", alice);
    println!("bob: {}", bob);
    assert_eq!(alice, alice_starting_lamports - amount);
    assert_eq!(bob, bob_starting_lamports + amount);

    println!("Number of cycles: {}", report.total_instruction_count());
}
