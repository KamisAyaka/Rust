use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{
        close_account, transfer_checked, CloseAccount, Mint, TokenAccount, TokenInterface,
        TransferChecked,
    },
};

use crate::Offer;

/// 撤销报价的账户结构
///
/// 允许报价创建者撤销未成交的报价，并取回托管的代币
#[derive(Accounts)]
#[instruction(offer_id: u64)]
pub struct CancelOffer<'info> {
    /// 报价创建者（必须是签名者）
    #[account(mut)]
    pub maker: Signer<'info>,

    /// 代币A的Mint账户
    pub token_mint_a: InterfaceAccount<'info, Mint>,

    /// 报价创建者的代币A账户（接收退回的代币）
    #[account(
        mut,
        associated_token::mint = token_mint_a,
        associated_token::authority = maker,
        associated_token::token_program = token_program
    )]
    pub maker_token_account_a: Box<InterfaceAccount<'info, TokenAccount>>,

    /// 报价状态账户
    #[account(
        mut,
        has_one = maker @ crate::error::ErrorCode::NotMaker,
        has_one = token_mint_a @ crate::error::ErrorCode::WrongTokenMint,
        seeds = [b"offer", maker.key().as_ref(), offer_id.to_le_bytes().as_ref()],
        bump = offer.bump,
    )]
    pub offer: Account<'info, Offer>,

    /// 代币A的托管账户
    #[account(
        mut,
        associated_token::mint = token_mint_a,
        associated_token::authority = offer,
        associated_token::token_program = token_program
    )]
    pub vault: InterfaceAccount<'info, TokenAccount>,

    /// 系统程序
    pub system_program: Program<'info, System>,
    /// 代币程序接口
    pub token_program: Interface<'info, TokenInterface>,
    /// 关联代币程序
    pub associated_token_program: Program<'info, AssociatedToken>,
}

/// 撤销报价主函数
///
/// 验证权限后，将托管的代币转回给创建者，并标记报价为已取消
///
/// # 参数
/// * `context` - 指令上下文，包含所有相关账户
///
/// # 返回值
/// * `Result<()>` - 成功返回Ok，失败返回错误信息
pub fn cancel_offer(context: Context<CancelOffer>) -> Result<()> {
    // 检查报价是否已经被取消
    require!(!context.accounts.offer.is_cancelled, crate::error::ErrorCode::OfferAlreadyCancelled);
    
    // 将托管的代币转回给创建者
    refund_to_maker(&context)?;

    // 关闭托管账户并将租金返还给创建者
    close_vault_transfer_to_maker(&context)?;

    // 标记报价为已取消
    let offer = &mut context.accounts.offer;
    offer.is_cancelled = true;

    Ok(())
}

/// 将托管的代币退还给报价创建者
///
/// # 参数
/// * `context` - 指令上下文
///
/// # 返回值
/// * `Result<()>` - 操作结果
fn refund_to_maker(context: &Context<CancelOffer>) -> Result<()> {
    let seeds = &[
        b"offer",
        context.accounts.maker.to_account_info().key.as_ref(),
        &context.accounts.offer.offer_id.to_le_bytes()[..],
        &[context.accounts.offer.bump],
    ];
    let signer_seeds = [&seeds[..]];

    let accounts = TransferChecked {
        from: context.accounts.vault.to_account_info(),
        to: context.accounts.maker_token_account_a.to_account_info(),
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
    )
}

/// 关闭托管账户并将租金返还给报价创建者
///
/// # 参数
/// * `context` - 指令上下文
///
/// # 返回值
/// * `Result<()>` - 操作结果
fn close_vault_transfer_to_maker(context: &Context<CancelOffer>) -> Result<()> {
    let seeds = &[
        b"offer",
        context.accounts.maker.to_account_info().key.as_ref(),
        &context.accounts.offer.offer_id.to_le_bytes()[..],
        &[context.accounts.offer.bump],
    ];
    let signer_seeds = [&seeds[..]];

    let accounts = CloseAccount {
        account: context.accounts.vault.to_account_info(),
        destination: context.accounts.maker.to_account_info(),
        authority: context.accounts.offer.to_account_info(),
    };

    let cpi_context = CpiContext::new_with_signer(
        context.accounts.token_program.to_account_info(),
        accounts,
        &signer_seeds,
    );

    close_account(cpi_context)
}
