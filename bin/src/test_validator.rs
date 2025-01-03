use {
    solana_sdk::{
        account::{Account, AccountSharedData},
        bpf_loader_upgradeable,
        commitment_config::CommitmentConfig,
        incinerator,
        instruction::{AccountMeta, Instruction},
        pubkey::Pubkey,
        rent::Rent,
        signature::Keypair,
        signer::Signer,
        transaction::Transaction,
    },
    solana_test_validator::{TestValidator, TestValidatorGenesis, UpgradeableProgramInfo},
    std::path::PathBuf,
};

pub struct ValidatorContext {
    pub test_validator: TestValidator,
    payer: Keypair,
}

impl ValidatorContext {
    async fn send_transaction(&self, instruction: Instruction) {
        let rpc_client = self.test_validator.get_async_rpc_client();
        let latest_blockhash = rpc_client.get_latest_blockhash().await.unwrap();

        let transaction = Transaction::new_signed_with_payer(
            &[instruction],
            Some(&self.payer.pubkey()),
            &[&self.payer],
            latest_blockhash,
        );

        rpc_client
            .send_and_confirm_transaction_with_spinner_and_commitment(
                &transaction,
                CommitmentConfig::confirmed(),
            )
            .await
            .unwrap();
    }

    pub async fn get_account(&self, account_id: &Pubkey) -> Option<Account> {
        self.test_validator
            .get_async_rpc_client()
            .get_account(account_id)
            .await
            .ok()
    }

    pub async fn close_counter(&self, key: &Pubkey) {
        self.send_transaction(Instruction::new_with_bytes(
            program::id(),
            &[1],
            vec![
                AccountMeta::new(*key, false),
                AccountMeta::new(incinerator::id(), false),
            ],
        ))
        .await;
    }

    pub async fn increment_counter(&self, key: &Pubkey) {
        self.send_transaction(Instruction::new_with_bytes(
            program::id(),
            &[0],
            vec![AccountMeta::new(*key, false)],
        ))
        .await;
    }

    pub async fn get_counter(&self, key: &Pubkey) -> u8 {
        let account = self
            .get_account(key)
            .await
            .expect("Counter account not found");

        *account.data.get(0).expect("Account data too small")
    }
}

pub async fn start_test_validator(keys: &[Pubkey]) -> ValidatorContext {
    let accounts = keys.iter().map(|key| {
        let lamports = Rent::default().minimum_balance(1);
        let account = AccountSharedData::new_data(lamports, &0u8, &program::id()).unwrap();
        (*key, account)
    });

    let programs = &[UpgradeableProgramInfo {
        program_id: program::id(),
        loader: bpf_loader_upgradeable::id(),
        program_path: PathBuf::from("target/deploy/program.so"),
        upgrade_authority: Pubkey::new_unique(),
    }];

    let (test_validator, payer) = TestValidatorGenesis::default()
        .add_accounts(accounts)
        .add_upgradeable_programs_with_path(programs)
        .start_async()
        .await;

    ValidatorContext {
        test_validator,
        payer,
    }
}
