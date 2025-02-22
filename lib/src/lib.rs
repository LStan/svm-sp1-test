use solana_bpf_loader_program::syscalls::create_program_runtime_environment_v1;
use solana_compute_budget::compute_budget::ComputeBudget;
use solana_program_runtime::loaded_programs::{BlockRelation, ForkGraph, ProgramCacheEntry};
use solana_sdk::{
    account::{AccountSharedData, ReadableAccount, WritableAccount},
    clock::Slot,
    feature_set::FeatureSet,
    fee::FeeStructure,
    hash::Hash,
    native_loader,
    pubkey::Pubkey,
    rent_collector::RentCollector,
    signature::{Keypair, Signer},
    system_instruction,
    transaction::{self, SanitizedTransaction, Transaction, TransactionError},
};

use solana_svm::{
    account_loader::CheckedTransactionDetails,
    transaction_processing_callback::TransactionProcessingCallback,
    transaction_processing_result::ProcessedTransaction,
    transaction_processor::{
        TransactionBatchProcessor, TransactionProcessingConfig, TransactionProcessingEnvironment,
    },
};
use std::{
    collections::{HashMap, HashSet},
    sync::{Arc, RwLock},
};

pub(crate) struct MockForkGraph {}

impl ForkGraph for MockForkGraph {
    fn relationship(&self, _a: Slot, _b: Slot) -> BlockRelation {
        BlockRelation::Unknown
    }
}

pub(crate) struct MockAccountLoader {
    pub account_shared_data: Arc<RwLock<HashMap<Pubkey, AccountSharedData>>>,
}

impl TransactionProcessingCallback for MockAccountLoader {
    fn get_account_shared_data(&self, pubkey: &Pubkey) -> Option<AccountSharedData> {
        self.account_shared_data
            .read()
            .unwrap()
            .get(pubkey)
            .cloned()
    }

    fn account_matches_owners(&self, account: &Pubkey, owners: &[Pubkey]) -> Option<usize> {
        self.get_account_shared_data(account)
            .and_then(|account| owners.iter().position(|key| account.owner().eq(key)))
    }

    fn add_builtin_account(&self, name: &str, program_id: &Pubkey) {
        let account_data = native_loader::create_loadable_account_with_fields(name, (5000, 0));
        self.account_shared_data
            .write()
            .unwrap()
            .insert(*program_id, account_data);
    }
}

pub fn transfer_sols(
    alice_starting_lamports: u64,
    bob_starting_lamports: u64,
    amount: u64,
) -> Result<(u64, u64), TransactionError> {
    let alice = Keypair::new();
    let bob = Keypair::new();
    let alice_pubkey = alice.pubkey();
    let bob_pubkey = bob.pubkey();

    let mut alice_account = AccountSharedData::default();
    alice_account.set_lamports(alice_starting_lamports);
    alice_account.set_owner(solana_sdk::system_program::id());
    let mut bob_account = AccountSharedData::default();
    bob_account.set_lamports(bob_starting_lamports);
    bob_account.set_owner(solana_sdk::system_program::id());

    let mut account_shared_data = HashMap::<Pubkey, AccountSharedData>::new();
    account_shared_data.insert(alice_pubkey, alice_account);
    account_shared_data.insert(bob_pubkey, bob_account);

    let account_loader = MockAccountLoader {
        account_shared_data: Arc::new(RwLock::new(account_shared_data)),
    };

    let fork_graph = Arc::new(RwLock::new(MockForkGraph {}));

    let processor = TransactionBatchProcessor::<MockForkGraph>::new(
        /* slot */ 1,
        /* epoch */ 1,
        Arc::downgrade(&fork_graph),
        Some(Arc::new(
            create_program_runtime_environment_v1(
                &FeatureSet::all_enabled(),
                &ComputeBudget::default(),
                false,
                false,
            )
            .unwrap(),
        )),
        None,
    );

    processor.add_builtin(
        &account_loader,
        solana_system_program::id(),
        "system_program",
        ProgramCacheEntry::new_builtin(
            0,
            b"system_program".len(),
            solana_system_program::system_processor::Entrypoint::vm,
        ),
    );

    let transaction = Transaction::new_with_payer(
        &[system_instruction::transfer(
            &alice_pubkey,
            &bob_pubkey,
            amount,
        )],
        Some(&alice_pubkey),
    );

    let mut svm_transactions: Vec<SanitizedTransaction> = Vec::new();

    svm_transactions.push(
        SanitizedTransaction::try_from_legacy_transaction(transaction, &HashSet::new()).unwrap(),
    );

    let fee_structure = FeeStructure::default();
    let rent_collector = RentCollector::default();

    // let processing_environment = TransactionProcessingEnvironment::default();
    let processing_environment = TransactionProcessingEnvironment {
        blockhash: Hash::default(),
        blockhash_lamports_per_signature: fee_structure.lamports_per_signature,
        epoch_total_stake: 0,
        feature_set: Arc::new(FeatureSet::all_enabled()),
        fee_lamports_per_signature: fee_structure.lamports_per_signature,
        rent_collector: Some(&rent_collector),
    };

    let processing_config = TransactionProcessingConfig {
        compute_budget: Some(ComputeBudget::default()),
        ..Default::default()
    };

    let results = processor.load_and_execute_sanitized_transactions(
        &account_loader,
        &svm_transactions,
        get_transaction_check_results(svm_transactions.len(), fee_structure.lamports_per_signature),
        &processing_environment,
        &processing_config,
    );

    let result = results.processing_results[0]
        .as_ref()
        .map_err(|e| e.clone())?;

    match result {
        ProcessedTransaction::Executed(executed_tx) => {
            let accounts = &executed_tx.loaded_transaction.accounts;
            Ok((accounts[0].1.lamports(), accounts[1].1.lamports()))
        }
        ProcessedTransaction::FeesOnly(details) => Err(details.load_error.clone()),
    }
}

pub(crate) fn get_transaction_check_results(
    len: usize,
    lamports_per_signature: u64,
) -> Vec<transaction::Result<CheckedTransactionDetails>> {
    vec![transaction::Result::Ok(CheckedTransactionDetails::new(None, lamports_per_signature)); len]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transfer_sols() {
        let alice_starting_lamports = 1_000_000;
        let bob_starting_lamports = 1_000_000;
        let amount = 666;
        let result = transfer_sols(alice_starting_lamports, bob_starting_lamports, amount);
        assert!(result.is_ok());
        let (alice, bob) = result.unwrap();
        assert_eq!(alice, alice_starting_lamports - amount);
        assert_eq!(bob, bob_starting_lamports + amount);
    }
}
