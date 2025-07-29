use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{Mint, TokenAccount, TokenInterface},
};

use crate::{Offer, ANCHOR_DISCRIMINATOR};

use super::transfer_tokens;

/// 创建报价的账户结构
/// 包含创建者、代币信息、资金库及系统程序等账户
/// 使用Anchor框架的Accounts宏自动生成账户验证代码
#[derive(Accounts)]
/// 指令参数包含报价ID
#[instruction(id: u64)]
pub struct MakeOffer<'info> {
    /// 交易发起人账户（签名者）
    /// mut表示该账户数据可变
    #[account(mut)]
    pub maker: Signer<'info>,

    /// 代币A的Mint账户
    /// mint::token_program指定使用的代币程序
    #[account(mint::token_program = token_program)]
    pub token_mint_a: InterfaceAccount<'info, Mint>,

    /// 代币B的Mint账户
    /// 同样指定使用的代币程序
    #[account(mint::token_program = token_program)]
    pub token_mint_b: InterfaceAccount<'info, Mint>,

    /// 发起人的代币A账户
    /// 关联到token_mint_a，由maker管理，使用指定的token_program
    #[account(
        mut,
        associated_token::mint = token_mint_a,
        associated_token::authority = maker,
        associated_token::token_program = token_program
    )]
    pub maker_token_account_a: InterfaceAccount<'info, TokenAccount>,

    /// 报价存储账户
    /// init表示该账户将被初始化
    /// seeds指定PDA种子，bump表示自动处理nonce
    #[account(
        init,
        payer = maker,
        space = ANCHOR_DISCRIMINATOR + Offer::INIT_SPACE,
        seeds = [b"offer", maker.key().as_ref(), id.to_le_bytes().as_ref()],
        bump
    )]
    pub offer: Account<'info, Offer>,

    /// 资金库账户
    /// 初始化为代币A的关联账户，由offer账户管理
    #[account(
        init,
        payer = maker,
        associated_token::mint = token_mint_a,
        associated_token::authority = offer,
        associated_token::token_program = token_program
    )]
    pub vault: InterfaceAccount<'info, TokenAccount>,

    /// 系统程序账户
    pub system_program: Program<'info, System>,
    
    /// 代币程序接口
    pub token_program: Interface<'info, TokenInterface>,
    
    /// 关联代币程序
    pub associated_token_program: Program<'info, AssociatedToken>,
}

/// 将报价代币转入资金库
/// @param context 上下文包含所有必要账户
/// @param token_a_offered_amount 要转入的代币数量
/// @return Result<()> 操作结果
pub fn send_offered_tokens_to_vault(
    context: &Context<MakeOffer>,
    token_a_offered_amount: u64,
) -> Result<()> {
    transfer_tokens(
        &context.accounts.maker_token_account_a,
        &context.accounts.vault,
        &token_a_offered_amount,
        &context.accounts.token_mint_a,
        &context.accounts.maker,
        &context.accounts.token_program,
    )
}

/// 保存报价信息到链上
/// @param context 上下文包含offer账户
/// @param id 报价ID
/// @param token_b_wanted_amount 需要的代币B数量
/// @return Result<()> 操作结果
pub fn save_offer(context: Context<MakeOffer>, id: u64, token_b_wanted_amount: u64) -> Result<()> {
    context.accounts.offer.set_inner(Offer {
        offer_id: (id),
        maker: (context.accounts.maker.key()),
        token_mint_a: (context.accounts.token_mint_a.key()),
        token_mint_b: (context.accounts.token_mint_b.key()),
        token_b_wanted_amount: (token_b_wanted_amount),
        bump: (context.bumps.offer),
        is_cancelled: false,
    });
    Ok(())
}
