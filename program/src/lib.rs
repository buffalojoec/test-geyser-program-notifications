//! Simple counter program, that can also close accounts.

use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    program_error::ProgramError,
    pubkey::Pubkey,
    system_program,
};

solana_program::declare_id!("Counter111111111111111111111111111111111111");

solana_program::entrypoint!(process);

pub fn process(_program_id: &Pubkey, accounts: &[AccountInfo], input: &[u8]) -> ProgramResult {
    match input.first() {
        Some(&0) => {
            // Increment the counter.
            let account = accounts.first().ok_or(ProgramError::NotEnoughAccountKeys)?;
            let mut data = account.try_borrow_mut_data()?;
            let value = data.get_mut(0).ok_or(ProgramError::InvalidAccountData)?;
            *value = value.saturating_add(1);
            Ok(())
        }
        Some(&1) => {
            // Close the account.
            let mut accounts_iter = &mut accounts.iter();

            let account = next_account_info(&mut accounts_iter)?;
            let incinerator = next_account_info(&mut accounts_iter)?;

            account.realloc(0, true)?;
            account.assign(&system_program::id());

            **incinerator.lamports.borrow_mut() =
                incinerator.lamports().saturating_add(account.lamports());
            **account.lamports.borrow_mut() = 0;

            Ok(())
        }
        _ => Err(ProgramError::InvalidInstructionData),
    }
}
