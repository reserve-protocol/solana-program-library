//! Folio program utility functions (Reserve Protocol DTF)

use crate::error::GovernanceError;
use solana_program::instruction::{AccountMeta, Instruction};
use solana_program::program::invoke;
use solana_program::pubkey::Pubkey;
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    hash, pubkey,
};

/// Folio program
pub struct FolioProgram {}

impl FolioProgram {
    const FOLIO_PROGRAM_ID: Pubkey = pubkey!("n6sR7Eg5LMg5SGorxK9q3ZePHs9e8gjoQ7TgUW2YCaG");

    const REMAINING_ACCOUNTS_GROUP_SIZE: usize = 4;

    fn get_instruction_discriminator(instruction_name: &str) -> [u8; 8] {
        let preimage = format!("global:{}", instruction_name);
        let mut hasher = hash::Hasher::default();
        hasher.hash(preimage.as_bytes());
        let hash_result = hasher.result();
        let mut discriminator = [0u8; 8];
        discriminator.copy_from_slice(&hash_result.to_bytes()[..8]);
        discriminator
    }

    /// Receives the remaining accounts from the instruction (expect in proper order), doesn't do validation as it's done on the Folio Program side.
    /// Only thing it will validate is the actual program being called is the Folio Program.
    pub fn accrue_rewards<'a>(
        system_info: &AccountInfo<'a>,
        spl_token_info: &AccountInfo<'a>,
        // The caller of the instruction
        governing_token_owner_info: &AccountInfo<'a>,
        // Is the token account that has the total "staked" balance
        governing_token_holding_info: &AccountInfo<'a>,
        // The user's token owner record info (his staked balance)
        token_owner_record_info: &AccountInfo<'a>,
        // Accounts starting from the last next account info in parent function
        accounts: &[AccountInfo<'a>],
        // Accounts for the reward tokens (can be up to 4 x 4)
        reward_token_accounts: &[AccountInfo<'a>],
    ) -> ProgramResult {
        let accounts_iter = &mut accounts.iter();

        let folio_program_info = next_account_info(accounts_iter)?;

        if *folio_program_info.key != FolioProgram::FOLIO_PROGRAM_ID
            || !folio_program_info.executable
        {
            return Err(GovernanceError::InvalidFolioProgram.into());
        }

        let folio_owner = next_account_info(accounts_iter)?;
        let actor = next_account_info(accounts_iter)?;
        let folio = next_account_info(accounts_iter)?;
        let folio_reward_tokens = next_account_info(accounts_iter)?;
        let governing_token_mint = next_account_info(accounts_iter)?;

        // Won't do the extra user, for simplicity's sake

        let mut accounts = vec![
            system_info.clone(),
            spl_token_info.clone(),
            governing_token_owner_info.clone(),
            folio_owner.clone(),
            actor.clone(),
            folio.clone(),
            folio_reward_tokens.clone(),
            governing_token_mint.clone(),
            governing_token_holding_info.clone(),
            token_owner_record_info.clone(),
            // For user send the same as caller
            governing_token_owner_info.clone(),
            // For user's governance token account, send same as caller
            governing_token_owner_info.clone(),
        ];

        let mut account_metas = vec![
            AccountMeta::new_readonly(*system_info.key, false),
            AccountMeta::new_readonly(*spl_token_info.key, false),
            AccountMeta::new(*governing_token_owner_info.key, true),
            AccountMeta::new_readonly(*folio_owner.key, false),
            AccountMeta::new_readonly(*actor.key, false),
            AccountMeta::new_readonly(*folio.key, false),
            AccountMeta::new_readonly(*folio_reward_tokens.key, false),
            AccountMeta::new_readonly(*governing_token_mint.key, false),
            AccountMeta::new_readonly(*governing_token_holding_info.key, false),
            AccountMeta::new_readonly(*token_owner_record_info.key, false),
            // For user send the same as caller
            AccountMeta::new(*governing_token_owner_info.key, true),
            // For user's governance token account, send same as caller
            AccountMeta::new(*governing_token_owner_info.key, true),
        ];

        // Remaining accounts for the instruction on Folio program
        let reward_token_accounts_iter = &mut reward_token_accounts.iter();
        for _ in 0..reward_token_accounts.len() / FolioProgram::REMAINING_ACCOUNTS_GROUP_SIZE {
            let reward_token_mint = next_account_info(reward_token_accounts_iter)?;
            let reward_info_for_token_mint = next_account_info(reward_token_accounts_iter)?;
            let folio_token_rewards_token_account = next_account_info(reward_token_accounts_iter)?;
            let reward_info_for_caller = next_account_info(reward_token_accounts_iter)?;

            accounts.push(reward_token_mint.clone());
            accounts.push(reward_info_for_token_mint.clone());
            accounts.push(folio_token_rewards_token_account.clone());
            accounts.push(reward_info_for_caller.clone());

            account_metas.push(AccountMeta::new_readonly(*reward_token_mint.key, false));
            account_metas.push(AccountMeta::new(*reward_info_for_token_mint.key, false));
            account_metas.push(AccountMeta::new_readonly(
                *folio_token_rewards_token_account.key,
                false,
            ));
            account_metas.push(AccountMeta::new(*reward_info_for_caller.key, false));
        }

        let data = FolioProgram::get_instruction_discriminator("accrue_rewards");

        let instruction = Instruction {
            program_id: *folio_program_info.key,
            accounts: account_metas,
            data: data.to_vec(),
        };

        invoke(&instruction, &accounts)?;

        Ok(())
    }
}
