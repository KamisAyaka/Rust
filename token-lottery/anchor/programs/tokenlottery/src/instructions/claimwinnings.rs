use crate::state::TokenLottery;
use crate::{constants::NAME, error::ErrorCode};
use anchor_lang::prelude::*;
use anchor_spl::{
    metadata::{Metadata, MetadataAccount},
    token_interface::{Mint, TokenAccount, TokenInterface},
};

/// 定义领取奖金的账户结构体
/// 需要验证用户拥有的NFT是否为获胜NFT，因此需要相关元数据账户。
#[derive(Accounts)]
pub struct ClaimWinnings<'info> {
    #[account(mut)]
    pub payer: Signer<'info>, // 领取奖金的用户账户

    // 彩票账户
    #[account(
        mut,
        seeds = [b"token_lottery".as_ref()],
        bump = token_lottery.bump,
    )]
    pub token_lottery: Account<'info, TokenLottery>,

    // 获胜彩票代币账户
    #[account(
        seeds = [token_lottery.winner.to_le_bytes().as_ref()],
         bump,
    )]
    pub ticket_mint: InterfaceAccount<'info, Mint>,

    // 集合代币账户
    #[account(
        seeds = [b"collection_mint".as_ref()],
        bump,
    )]
    pub collection_mint: InterfaceAccount<'info, Mint>,

    // 彩票元数据账户
    #[account(
        seeds=[b"metadata",token_metadata_program.key().as_ref(),ticket_mint.key().as_ref()],
        bump,
        seeds::program  = token_metadata_program.key(),
    )]
    pub ticket_metadata: Account<'info, MetadataAccount>,

    // 用户持有的彩票代币账户
    #[account(
        associated_token::mint = ticket_mint,
        associated_token::authority = payer,
        associated_token::token_program  = token_program,
    )]
    pub ticket_account: InterfaceAccount<'info, TokenAccount>,

    // 集合元数据账户
    #[account(
        seeds=[b"metadata",token_metadata_program.key().as_ref(),collection_mint.key().as_ref()],
        bump,
        seeds::program  = token_metadata_program.key(),
    )]
    pub collection_metadata: Account<'info, MetadataAccount>,

    pub token_metadata_program: Program<'info, Metadata>, // Metaplex元数据程序
    pub token_program: Interface<'info, TokenInterface>,  // 代币程序
}

/// 领取奖金
/// 获胜者调用此函数领取奖池奖金
///
/// 需要较多账户来验证用户是否拥有获胜的彩票NFT：
/// - token_lottery: 我们的彩票账户
/// - ticket_mint: 获胜彩票代币账户
/// - collection_mint: 集合代币账户
/// - ticket_metadata: 彩票元数据账户
/// - ticket_account: 用户持有的彩票代币账户
/// - collection_metadata: 集合元数据账户
/// - 各种程序账户
pub fn claim_winnings(ctx: Context<ClaimWinnings>) -> Result<()> {
    // 检查是否已经选出获胜者
    require!(
        ctx.accounts.token_lottery.winner_chosen,
        ErrorCode::WinnerNotChosen
    );

    // 检查彩票NFT是否属于正确的集合且已验证
    require!(
        ctx.accounts
            .ticket_metadata
            .collection
            .as_ref()
            .unwrap()
            .verified,
        ErrorCode::CollectionNotVerified
    );

    // 检查彩票NFT是否属于当前彩票集合
    require!(
        ctx.accounts
            .ticket_metadata
            .collection
            .as_ref()
            .unwrap()
            .key
            == ctx.accounts.collection_mint.key(),
        ErrorCode::IncorrectTicket
    );

    // 检查彩票NFT编号是否与获胜者编号一致
    let ticket_name = NAME.to_owned() + &ctx.accounts.token_lottery.winner.to_string();
    let metadata_name = ctx.accounts.ticket_metadata.name.replace("\u{0}", "");
    require!(metadata_name == ticket_name, ErrorCode::IncorrectTicket);

    // 检查用户是否持有该彩票NFT
    require!(ctx.accounts.ticket_account.amount > 0, ErrorCode::NoTicket);

    // 将奖池资金转移给获胜者
    **ctx
        .accounts
        .token_lottery
        .to_account_info()
        .lamports
        .borrow_mut() -= ctx.accounts.token_lottery.lottery_pot_amount;

    **ctx.accounts.payer.to_account_info().lamports.borrow_mut() +=
        ctx.accounts.token_lottery.lottery_pot_amount;

    // 清空奖池
    ctx.accounts.token_lottery.lottery_pot_amount = 0;

    Ok(())
}
