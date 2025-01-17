use anchor_lang::prelude::*;
use anchor_spl::token::{Token, TokenAccount};

use std::collections::HashMap;

declare_id!("Av142jsr7gKXsDWjVuY9oVz266vKWaqyeD8m1gdZHr4c");

#[program]
pub mod nft_wechat_binding {
    use super::*;

    /// Bind WeChat ID to NFT holder's address
    pub fn bind_wechat_id<'info>(
        ctx: Context<'_, '_, '_, 'info, BindWechatId<'info>>,
        wechat_id: String,
    ) -> Result<()> {
        let account = &mut ctx.accounts.binding_account;

        // Verify user owns the NFT
        require!(ctx.accounts.token_account.amount > 0, CustomError::NoNFT);

        // Bind WeChat ID
        account.owner = *ctx.accounts.user.key;
        account.wechat_id = wechat_id.clone();
        account.timestamp = Clock::get()?.unix_timestamp;
        account.nft_mint = ctx.accounts.token_account.mint;

        // Register the binding record
        let registry = &mut ctx.accounts.binding_registry;
        registry.bindings.push(account.key());

        Ok(())
    }

    /// Query the binding status of a WeChat ID
    pub fn query_binding<'info>(
        ctx: Context<'_, '_, '_, 'info, QueryBinding<'info>>,
        wechat_id: String,
    ) -> Result<String> {
        let account = &ctx.accounts.binding_account;

        if account.wechat_id == wechat_id {
            Ok("Verified".to_string())
        } else {
            Ok("Not Verified".to_string())
        }
    }

    /// Global check to clean up invalid bindings
    pub fn global_check<'info>(
        ctx: Context<'_, '_, 'info, 'info, GlobalCheck<'info>>,
    ) -> Result<()> {
        let registry = &mut ctx.accounts.binding_registry;

        // Create a new list of valid bindings
        let mut updated_bindings = Vec::new();

        for binding_key in &registry.bindings {
            if let Some(binding_account_info) = ctx.remaining_accounts.iter().find(|acc| acc.key() == *binding_key) {
                if let Ok(token_account) = Account::<TokenAccount>::try_from(binding_account_info) {
                    if token_account.amount > 0 {
                        updated_bindings.push(*binding_key); // Retain valid binding
                    }
                }
            }
        }

        // Update the registry with valid bindings
        registry.bindings = updated_bindings;
        Ok(())
    }
}

#[derive(Accounts)]
pub struct BindWechatId<'info> {
    #[account(mut)]
    pub user: Signer<'info>,                             // User signing the transaction
    #[account(init, payer = user, space = 8 + 32 + 64 + 8 + 32)]
    pub binding_account: Account<'info, BindingAccount>, // Binding record account
    #[account(mut)]
    pub binding_registry: Account<'info, BindingRegistry>, // Global binding registry
    #[account(constraint = token_account.amount > 0 @ CustomError::NoNFT)]
    pub token_account: Account<'info, TokenAccount>,     // NFT Token account
    pub system_program: Program<'info, System>,          // System program
}

#[derive(Accounts)]
pub struct QueryBinding<'info> {
    #[account(mut)]
    pub binding_account: Account<'info, BindingAccount>, // Binding record account
}

#[derive(Accounts)]
pub struct GlobalCheck<'info> {
    #[account(mut, has_one = admin)]
    pub binding_registry: Account<'info, BindingRegistry>, // Global binding registry
    pub admin: Signer<'info>,                              // Admin signing the transaction
    pub token_program: Program<'info, Token>,              // Token program
    #[account(address = spl_token::id())]
    pub token_account_program: UncheckedAccount<'info>,    // Token program account
}

#[account]
pub struct BindingAccount {
    pub owner: Pubkey,      // Owner's address
    pub wechat_id: String,  // WeChat ID
    pub timestamp: i64,     // Binding timestamp
    pub nft_mint: Pubkey,   // Mint address of the NFT
}

#[account]
pub struct BindingRegistry {
    pub bindings: Vec<Pubkey>, // All binding accounts' PublicKeys
    pub admin: Pubkey,         // Admin's address
}

#[error_code]
pub enum CustomError {
    #[msg("User does not hold the required NFT")]
    NoNFT,
}
