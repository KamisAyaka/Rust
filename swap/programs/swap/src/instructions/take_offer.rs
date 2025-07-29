use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{
        close_account, transfer_checked, CloseAccount, Mint, TokenAccount, TokenInterface,
        TransferChecked,
    },
};

use crate::Offer;

use super::transfer_tokens;

#[derive(Accounts)]
pub struct TakeOffer<'info> {
    /// 交易执行者（接受报价的人），需提供签名
    #[account(mut)]
    pub taker: Signer<'info>,

    /// 报价创建者账户，需为系统账户
    #[account(mut)]
    pub maker: SystemAccount<'info>,

    /// 代币A的Mint账户
    pub token_mint_a: InterfaceAccount<'info, Mint>,
    /// 代币B的Mint账户
    pub token_mint_b: InterfaceAccount<'info, Mint>,

    /// 交易执行者代币A的关联账户（如果不存在则自动创建）
    #[account(
        init_if_needed,
        payer = taker,
        associated_token::mint = token_mint_a,
        associated_token::authority = taker,
        associated_token::token_program = token_program
    )]
    pub taker_token_account_a: Box<InterfaceAccount<'info, TokenAccount>>,

    /// 交易执行者代币B的关联账户（需为可变账户）
    #[account(
        mut,
        associated_token::mint = token_mint_b,
        associated_token::authority = taker,
        associated_token::token_program = token_program
    )]
    pub taker_token_account_b: Box<InterfaceAccount<'info, TokenAccount>>,

    /// 报价创建者代币B的关联账户（如果不存在则自动创建）
    #[account(
        init_if_needed,
        payer = taker,
        associated_token::mint = token_mint_b,
        associated_token::authority = maker,
        associated_token::token_program = token_program
    )]
    pub maker_token_account_b: Box<InterfaceAccount<'info, TokenAccount>>,

    /// 报价状态账户（执行后自动关闭并返还给创建者）
    #[account(
        mut,
        close = maker,
        has_one = maker,
        has_one = token_mint_a,
        has_one = token_mint_b,
        seeds = [b"offer", maker.key().as_ref(), offer.offer_id.to_le_bytes().as_ref()],
        bump = offer.bump
    )]
    pub offer: Account<'info, Offer>,

    /// 代币A的托管账户（由报价创建者管理）
    #[account(
        mut,
        associated_token::mint = token_mint_a,
        associated_token::authority = offer,
        associated_token::token_program = token_program
    )]
    vault: InterfaceAccount<'info, TokenAccount>,

    /// 系统程序账户
    pub system_program: Program<'info, System>,
    /// 代币程序账户
    pub token_program: Interface<'info, TokenInterface>,
    /// 关联代币程序账户
    pub associated_token_program: Program<'info, AssociatedToken>,
}

/// 将期望的代币B发送给报价创建者
///
/// # 参数
/// - `context`: 指令执行上下文，包含所有必要账户
///
/// # 返回值
/// - `Result<()>`: 操作成功返回Ok，失败返回错误信息
pub fn send_wanted_tokens_to_maker(context: &Context<TakeOffer>) -> Result<()> {
    transfer_tokens(
        &context.accounts.taker_token_account_b,
        &context.accounts.maker_token_account_b,
        &context.accounts.offer.token_b_wanted_amount,
        &context.accounts.token_mint_b,
        &context.accounts.taker,
        &context.accounts.token_program,
    )
}

/// 从托管账户提取代币A并关闭账户
///
/// 执行两步操作：
/// 1. 将托管账户中的代币A转移到交易执行者账户
/// 2. 关闭托管账户并将租金返还给执行者
///
/// # 参数
/// - `context`: 指令执行上下文，包含所有必要账户
///
/// # 返回值
/// - `Result<()>`: 操作成功返回Ok，失败返回错误信息
pub fn withdraw_and_close_vault(context: &Context<TakeOffer>) -> Result<()> {
    let seeds = &[
        b"offer",
        context.accounts.maker.to_account_info().key.as_ref(),
        &context.accounts.offer.offer_id.to_le_bytes()[..],
        &[context.accounts.offer.bump],
    ];
    let signer_seeds = [&seeds[..]];
    let accounts = TransferChecked {
        from: context.accounts.vault.to_account_info(),
        to: context.accounts.taker_token_account_a.to_account_info(),
        mint: context.accounts.token_mint_a.to_account_info(),
        authority: context.accounts.offer.to_account_info(),
    };

    let cpi_context = CpiContext::new_with_signer(
        context.accounts.token_program.to_account_info(),
        accounts,
        &signer_seeds,
    );
    transfer_checked(
        cpi_context,
        context.accounts.vault.amount,
        context.accounts.token_mint_a.decimals,
    )?;

    let accounts = CloseAccount {
        account: context.accounts.vault.to_account_info(),
        destination: context.accounts.taker.to_account_info(),
        authority: context.accounts.offer.to_account_info(),
    };

    let cpi_context = CpiContext::new_with_signer(
        context.accounts.token_program.to_account_info(),
        accounts,
        &signer_seeds,
    );
    close_account(cpi_context)
}
