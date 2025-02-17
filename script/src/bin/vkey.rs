use sp1_sdk::{include_elf, HashableKey, Prover, ProverClient};

/// The ELF (executable and linkable format) file for the Succinct RISC-V zkVM.
pub const SVM_SP1_TEST_ELF: &[u8] = include_elf!("svm-sp1-test-program");

fn main() {
    let prover = ProverClient::builder().cpu().build();
    let (_, vk) = prover.setup(SVM_SP1_TEST_ELF);
    println!("{}", vk.bytes32());
}
