use anchor_lang::prelude::*;
use anchor_spl::token::{Token, TokenAccount};

declare_id!("Av142jsr7gKXsDWjVuY9oVz266vKWaqyeD8m1gdZHr4c");

#[program]
pub mod nft_wechat_binding {
    use super::*;

    /// 绑定微信ID到NFT持有地址
    pub fn bind_wechat_id(ctx: Context<BindWechatId>, wechat_id: String) -> Result<()> {
        let account = &mut ctx.accounts.binding_account;

        // 验证用户持有NFT
        require!(ctx.accounts.token_account.amount > 0, CustomError::NoNFT);

        // 绑定微信ID
        account.owner = *ctx.accounts.user.key;
        account.wechat_id = wechat_id.clone();
        account.timestamp = Clock::get()?.unix_timestamp;
        account.nft_mint = ctx.accounts.token_account.mint;

        // 注册绑定记录
        let registry = &mut ctx.accounts.binding_registry;
        registry.bindings.push(account.key());

        Ok(())
    }

    /// 查询微信ID绑定的状态
    pub fn query_binding(ctx: Context<QueryBinding>, wechat_id: String) -> Result<String> {
        let account = &ctx.accounts.binding_account;

        if account.wechat_id == wechat_id {
            Ok("已认证".to_string())
        } else {
            Ok("未认证".to_string())
        }
    }

    /// 全局检查功能
    /// 删除没有持有NFT的绑定记录
    pub fn global_check(ctx: Context<GlobalCheck>) -> Result<()> {
        let registry = &mut ctx.accounts.binding_registry;

        // 遍历所有绑定记录
        registry.bindings.retain(|binding_key| {
            // 解引用 binding_key
            let binding_account = ctx.remaining_accounts.iter().find(|acc| acc.key() == *binding_key);

            if let Some(binding_account) = binding_account {
                let token_account_info = ctx.remaining_accounts.iter().find(|acc| acc.owner == &spl_token::id());
                if let Some(token_account_info) = token_account_info {
                    // 将 token_account_info 转换为 TokenAccount
                    if let Ok(token_account) = Account::<TokenAccount>::try_from(token_account_info) {
                        if token_account.amount > 0 {
                            return true; // 持有NFT，保留绑定
                        }
                    }
                }
            }

            false // 未找到有效绑定，删除
        });

        Ok(())
    }
}

#[derive(Accounts)]
pub struct BindWechatId<'info> {
    #[account(mut)]
    pub user: Signer<'info>,                             // 用户签名者
    #[account(init, payer = user, space = 8 + 32 + 64 + 8 + 32)]
    pub binding_account: Account<'info, BindingAccount>, // 绑定记录账户
    #[account(mut)]
    pub binding_registry: Account<'info, BindingRegistry>, // 全局绑定注册表
    #[account(constraint = token_account.amount > 0 @ CustomError::NoNFT)]
    pub token_account: Account<'info, TokenAccount>,     // NFT Token账户
    pub system_program: Program<'info, System>,          // 系统程序
}

#[derive(Accounts)]
pub struct QueryBinding<'info> {
    #[account(mut)]
    pub binding_account: Account<'info, BindingAccount>, // 绑定记录账户
}

#[derive(Accounts)]
pub struct GlobalCheck<'info> {
    #[account(mut, has_one = admin)]
    pub binding_registry: Account<'info, BindingRegistry>, // 全局绑定注册表
    pub admin: Signer<'info>,                              // 管理员签名
    pub token_program: Program<'info, Token>,              // Token程序
    #[account(address = spl_token::id())]
    pub token_account_program: UncheckedAccount<'info>,    // Token程序账户
}

#[account]
pub struct BindingAccount {
    pub owner: Pubkey,      // 持有者地址
    pub wechat_id: String,  // 微信ID
    pub timestamp: i64,     // 绑定时间戳
    pub nft_mint: Pubkey,   // NFT的Mint地址
}

#[account]
pub struct BindingRegistry {
    pub bindings: Vec<Pubkey>, // 所有绑定账户的PublicKey
    pub admin: Pubkey,         // 管理员地址
}

#[error_code]
pub enum CustomError {
    #[msg("User does not hold the required NFT")]
    NoNFT,
}
