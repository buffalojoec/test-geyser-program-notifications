use {
    solana_client::{
        pubsub_client::{AccountSubscription, ProgramSubscription, PubsubClient},
        rpc_config::{RpcAccountInfoConfig, RpcProgramAccountsConfig},
    },
    solana_sdk::{commitment_config::CommitmentConfig, pubkey::Pubkey},
};

enum Subscription {
    Account(AccountSubscription),
    Program(ProgramSubscription),
}

pub struct SubscriptionContext {
    subscription: Subscription,
}

impl SubscriptionContext {
    pub fn account(pubsub_url: &str, pubkey: &Pubkey) -> Self {
        let subscription = PubsubClient::account_subscribe(
            pubsub_url,
            pubkey,
            Some(RpcAccountInfoConfig {
                commitment: Some(CommitmentConfig::processed()),
                ..Default::default()
            }),
        )
        .expect("Failed to create account subscription");
        Self {
            subscription: Subscription::Account(subscription),
        }
    }

    pub fn program(pubsub_url: &str, program_id: &Pubkey) -> Self {
        let subscription = PubsubClient::program_subscribe(
            &pubsub_url,
            program_id,
            Some(RpcProgramAccountsConfig {
                account_config: RpcAccountInfoConfig {
                    commitment: Some(CommitmentConfig::processed()),
                    ..Default::default()
                },
                ..Default::default()
            }),
        )
        .expect("Failed to create program subscription");
        Self {
            subscription: Subscription::Program(subscription),
        }
    }

    pub fn len(&self) -> usize {
        match self.subscription {
            Subscription::Account(ref account) => account.1.len(),
            Subscription::Program(ref program) => program.1.len(),
        }
    }

    pub fn shutdown(&mut self) {
        match self.subscription {
            Subscription::Account(ref mut account) => account.0.shutdown().unwrap(),
            Subscription::Program(ref mut program) => program.0.shutdown().unwrap(),
        }
    }
}
