use crate::state::TokenLottery;
use crate::{
    constants::{NAME, SYMBOL, URI},
    error::ErrorCode,
};
use anchor_lang::prelude::*;
use anchor_lang::system_program;
use anchor_spl::metadata::{
    create_master_edition_v3, create_metadata_accounts_v3, mpl_token_metadata::types::DataV2,
    set_and_verify_sized_collection_item, CreateMasterEditionV3, CreateMetadataAccountsV3,
    SetAndVerifySizedCollectionItem,
};
use anchor_spl::{
    associated_token::AssociatedToken,
    metadata::Metadata,
    token_interface::{mint_to, Mint, MintTo, TokenAccount, TokenInterface},
};

/// 定义购买彩票的账户结构体
/// 购买彩票涉及创建新的NFT，因此需要声明所有相关的元数据账户。
#[derive(Accounts)]
pub struct BuyTicket<'info> {
    #[account(mut)]
    pub payer: Signer<'info>, // 购买彩票的用户账户

    // 彩票账户
    #[account(
        mut,
        seeds = [b"token_lottery".as_ref()],
        bump = token_lottery.bump,
    )]
    pub token_lottery: Account<'info, TokenLottery>,

    // 新彩票代币账户
    #[account(
        init,
        payer = payer,
        seeds = [token_lottery.total_tickets.to_le_bytes().as_ref()],
        bump,
        mint::decimals = 0,
        mint::authority = collection_mint,
        mint::freeze_authority = collection_mint,
        mint::token_program  = token_program,
    )]
    pub ticket_mint: InterfaceAccount<'info, Mint>,

    // 用户的关联代币账户，用于接收彩票
    #[account(
        init,
        payer = payer,
        associated_token::mint = ticket_mint,
        associated_token::authority = payer,
        associated_token::token_program  = token_program,
    )]
    pub destination: InterfaceAccount<'info, TokenAccount>,

    // 彩票元数据账户
    #[account(
        mut,
        seeds = [
            b"metadata",
            token_metadata_program.key().as_ref(),
            ticket_mint.key().as_ref()
        ],
        bump,
        seeds::program  = token_metadata_program.key(),
    )]
    /// CHECK: This account is checked by the metadata smart contract
    pub ticket_metadata: UncheckedAccount<'info>,

    // 彩票主版本账户
    #[account(
        mut,
        seeds = [
            b"metadata",
            token_metadata_program.key().as_ref(),
            ticket_mint.key().as_ref(),
            b"edition"
        ],
        bump,
        seeds::program  = token_metadata_program.key(),
    )]
    /// CHECK: This account is checked by the metadata smart contract
    pub ticket_master_edition: UncheckedAccount<'info>,

    // 集合元数据账户
    #[account(
        mut,
        seeds = [
            b"metadata",
            token_metadata_program.key().as_ref(),
            collection_mint.key().as_ref()
        ],
        bump,
        seeds::program  = token_metadata_program.key(),
    )]
    /// CHECK: This account is checked by the metadata smart contract
    pub collection_metadata: UncheckedAccount<'info>,

    // 集合主版本账户
    #[account(
        mut,
        seeds = [
            b"metadata",
            token_metadata_program.key().as_ref(),
            collection_mint.key().as_ref(),
            b"edition"
        ],
        bump,
        seeds::program  = token_metadata_program.key(),
    )]
    /// CHECK: This account is checked by the metadata smart contract
    pub collection_master_edition: UncheckedAccount<'info>,

    // 集合代币账户
    #[account(
        mut,
        seeds = [b"collection_mint".as_ref()],
        bump,
    )]
    pub collection_mint: InterfaceAccount<'info, Mint>,

    pub associated_token_program: Program<'info, AssociatedToken>, // 关联代币程序
    pub token_program: Interface<'info, TokenInterface>,           // 代币程序
    pub token_metadata_program: Program<'info, Metadata>,          // Metaplex元数据程序
    pub system_program: Program<'info, System>,                    // 系统程序
    pub rent: Sysvar<'info, Rent>,                                 // 租金变量
}

/// 购买彩票
/// 用户调用此函数购买一张彩票NFT
///
/// 购买彩票涉及创建一个新的NFT，因此需要大量账户：
/// - ticket_mint: 新彩票代币账户
/// - destination: 用户的关联代币账户，用于接收彩票
/// - ticket_metadata: 新彩票NFT的元数据账户
/// - ticket_master_edition: 新彩票NFT的主版本账户
/// - collection_metadata: 集合元数据账户
/// - collection_master_edition: 集合主版本账户
/// - collection_mint: 集合代币账户
/// - 各种程序账户（token_metadata_program, associated_token_program等）
pub fn buy_ticket(ctx: Context<BuyTicket>) -> Result<()> {
    // 获取当前区块链时间（slot）
    let clock = Clock::get()?;
    // 生成彩票NFT名称，格式为"Token Lottery Ticket #<编号>"
    let ticket_name = NAME.to_owned()
        + ctx
            .accounts
            .token_lottery
            .total_tickets
            .to_string()
            .as_str();

    // 检查当前是否在彩票销售时间内
    if clock.slot < ctx.accounts.token_lottery.start_time
        || clock.slot > ctx.accounts.token_lottery.end_time
    {
        return Err(ErrorCode::LotteryNotOpen.into());
    }

    // 从购买者账户转移资金到彩票账户（支付彩票费用）
    system_program::transfer(
        CpiContext::new(
            ctx.accounts.system_program.to_account_info(),
            system_program::Transfer {
                from: ctx.accounts.payer.to_account_info(),
                to: ctx.accounts.token_lottery.to_account_info(),
            },
        ),
        ctx.accounts.token_lottery.ticket_price,
    )?;

    // 定义签名种子
    let signer_seeds: &[&[&[u8]]] = &[&[b"collection_mint".as_ref(), &[ctx.bumps.collection_mint]]];

    // 铸造一张新的彩票代币到用户账户
    mint_to(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            MintTo {
                mint: ctx.accounts.ticket_mint.to_account_info(),
                to: ctx.accounts.destination.to_account_info(),
                authority: ctx.accounts.collection_mint.to_account_info(),
            },
            &signer_seeds,
        ),
        1,
    )?;

    // 为新铸造的彩票代币创建元数据
    create_metadata_accounts_v3(
        CpiContext::new_with_signer(
            ctx.accounts.token_metadata_program.to_account_info(),
            CreateMetadataAccountsV3 {
                metadata: ctx.accounts.ticket_metadata.to_account_info(),
                mint: ctx.accounts.ticket_mint.to_account_info(),
                mint_authority: ctx.accounts.collection_mint.to_account_info(),
                payer: ctx.accounts.payer.to_account_info(),
                update_authority: ctx.accounts.collection_mint.to_account_info(),
                system_program: ctx.accounts.system_program.to_account_info(),
                rent: ctx.accounts.rent.to_account_info(),
            },
            signer_seeds,
        ),
        DataV2 {
            name: ticket_name,
            symbol: SYMBOL.to_string(),
            uri: URI.to_string(),
            seller_fee_basis_points: 0,
            creators: None,
            collection: None,
            uses: None,
        },
        true,
        true,
        None,
    )?;

    msg!("Creating Master Edition account");
    // 为彩票代币创建主版本账户，使其成为NFT
    create_master_edition_v3(
        CpiContext::new_with_signer(
            ctx.accounts.token_metadata_program.to_account_info(),
            CreateMasterEditionV3 {
                payer: ctx.accounts.payer.to_account_info(),
                mint: ctx.accounts.ticket_mint.to_account_info(),
                edition: ctx.accounts.ticket_master_edition.to_account_info(),
                mint_authority: ctx.accounts.collection_mint.to_account_info(),
                update_authority: ctx.accounts.collection_mint.to_account_info(),
                metadata: ctx.accounts.ticket_metadata.to_account_info(),
                token_program: ctx.accounts.token_program.to_account_info(),
                system_program: ctx.accounts.system_program.to_account_info(),
                rent: ctx.accounts.rent.to_account_info(),
            },
            &signer_seeds,
        ),
        Some(0),
    )?;

    // 将彩票NFT添加到集合中
    set_and_verify_sized_collection_item(
        CpiContext::new_with_signer(
            ctx.accounts.token_metadata_program.to_account_info(),
            SetAndVerifySizedCollectionItem {
                metadata: ctx.accounts.ticket_metadata.to_account_info(),
                collection_authority: ctx.accounts.collection_mint.to_account_info(),
                payer: ctx.accounts.payer.to_account_info(),
                update_authority: ctx.accounts.collection_mint.to_account_info(),
                collection_mint: ctx.accounts.collection_mint.to_account_info(),
                collection_metadata: ctx.accounts.collection_metadata.to_account_info(),
                collection_master_edition: ctx.accounts.collection_master_edition.to_account_info(),
            },
            &signer_seeds,
        ),
        None,
    )?;

    // 增加总票数
    ctx.accounts.token_lottery.total_tickets += 1;

    Ok(())
}
