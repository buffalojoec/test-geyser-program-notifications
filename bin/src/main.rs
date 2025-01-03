//! The program simply increments a counter in the account data.
//!
//! We'll subscribe to program account updates for two program-owned accounts.
//! The first account will continue as a program account.
//! The second account will be "closed" after a few increments.
//!
//! Only the account subscription can detect account closure.

mod pubsub;
mod test_validator;

use {crate::pubsub::SubscriptionContext, solana_sdk::pubkey::Pubkey};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let key1 = Pubkey::new_unique();
    let key2 = Pubkey::new_unique();

    // When the test validator goes out of scope, it will shut down.
    {
        let context = test_validator::start_test_validator(&[key1, key2]).await;
        let pubsub_url = context.test_validator.rpc_pubsub_url();

        // Subscribe to each account, as well as the program accounts.
        let client_account_1 = SubscriptionContext::account(&pubsub_url, &key1);
        let client_account_2 = SubscriptionContext::account(&pubsub_url, &key2);
        let client_program = SubscriptionContext::program(&pubsub_url, &program::id());

        // Start by incrementing each counter.
        for _ in 0..3 {
            println!("Incrementing account #1...");
            context.increment_counter(&key1).await;
            let counter = context.get_counter(&key1).await;
            println!("Counter #1: {}", counter);

            println!("Incrementing account #2...");
            context.increment_counter(&key2).await;
            let counter = context.get_counter(&key2).await;
            println!("Counter #2: {}", counter);
        }

        // Now close the second account.
        println!("Closing account #2...");
        context.close_counter(&key2).await;
        let account_2 = context.get_account(&key2).await;
        println!("Account #2: {:?}", account_2);
        assert!(account_2.is_none());

        // We should have the following messages:
        // * Account #1: 3: 3 increments
        // * Account #2: 4: 3 increments + 1 close
        // * Program   : 6: 3 increments * 2 accounts
        //
        // Note that the "close" is missing from the program messages.
        println!("Messages:");
        println!("Account #1: {}", client_account_1.len());
        println!("Account #2: {}", client_account_2.len());
        println!("Program   : {}", client_program.len());
        assert_eq!(client_account_1.len(), 3);
        assert_eq!(client_account_2.len(), 4);
        assert_eq!(client_program.len(), 6);

        [client_account_1, client_account_2, client_program]
    }
    .iter_mut()
    .for_each(|client| client.shutdown());

    println!("Shutting down... Please wait...");

    Ok(())
}
