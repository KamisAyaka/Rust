use anchor_lang::prelude::*;

use anchor_spl::metadata::{
    create_master_edition_v3, create_metadata_accounts_v3,
    mpl_token_metadata::types::{CollectionDetails, Creator, DataV2},
    sign_metadata, CreateMasterEditionV3, CreateMetadataAccountsV3, SignMetadata,
};
use anchor_spl::{
    associated_token::AssociatedToken,
    metadata::Metadata,
    token_interface::{mint_to, Mint, MintTo, TokenAccount, TokenInterface},
};

use crate::{constants::*, state::TokenLottery};
/// 定义初始化账户结构体
/// 在Solana中，每个程序函数都需要一个明确的账户结构体，
/// 描述函数执行所需的所有账户。
/// 这是Solana安全模型的核心部分 - 所有账户访问都必须预先声明和验证。
#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(mut)]
    pub payer: Signer<'info>, // 支付交易费用的账户，也是彩票管理员

    // 初始化TokenLottery账户
    #[account(
        init,
        payer = payer,
        space = 8 + TokenLottery::INIT_SPACE,
        seeds = [b"token_lottery".as_ref()],
        bump,
    )]
    pub token_lottery: Account<'info, TokenLottery>,
    pub system_program: Program<'info, System>, // Solana系统程序
}

/// 定义初始化彩票集合的账户结构体
/// 创建NFT需要与Metaplex Token Metadata程序交互，
/// 因此需要声明该程序所需的所有账户。
#[derive(Accounts)]
pub struct InitializeLottery<'info> {
    #[account(mut)]
    pub payer: Signer<'info>, // 支付交易费用的账户

    // 初始化集合代币账户
    #[account(
        init,
        payer = payer,
        mint::decimals = 0,
        mint::authority = collection_mint,
        mint::freeze_authority = collection_mint,
        seeds = [b"collection_mint".as_ref()],
        bump,
    )]
    pub collection_mint: InterfaceAccount<'info, Mint>,

    // 初始化集合代币的关联代币账户
    #[account(
        init,
        payer = payer,
        token::mint = collection_mint,
        token::authority = collection_token_account,
        seeds = [b"collection_associated_token".as_ref()],
        bump,
    )]
    pub collection_token_account: InterfaceAccount<'info, TokenAccount>,

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
    pub metadata: UncheckedAccount<'info>,

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
    pub master_edition: UncheckedAccount<'info>,

    pub token_metadata_program: Program<'info, Metadata>, // Metaplex元数据程序
    pub associated_token_program: Program<'info, AssociatedToken>, // 关联代币程序
    pub token_program: Interface<'info, TokenInterface>,  // 代币程序
    pub system_program: Program<'info, System>,           // 系统程序
    pub rent: Sysvar<'info, Rent>,                        // 租金变量
}

/// 初始化彩票配置
///
/// 为什么需要这么多账户？
/// 在Solana中，所有需要读取或写入的账户都必须在交易中显式声明。
/// 这是Solana安全模型的一部分，允许运行时进行并行处理优化和安全检查。
///
/// 参数:
/// - ctx: 上下文，包含所有需要的账户
/// - start: 开始时间（slot）
/// - end: 结束时间（slot）
/// - price: 每张彩票的价格（lamports）
pub fn initialize_config(ctx: Context<Initialize>, start: u64, end: u64, price: u64) -> Result<()> {
    // 设置账户的bump种子，用于PDA（Program Derived Address）验证
    ctx.accounts.token_lottery.bump = ctx.bumps.token_lottery;
    // 设置彩票开始和结束时间
    ctx.accounts.token_lottery.start_time = start;
    ctx.accounts.token_lottery.end_time = end;
    // 设置每张彩票的价格
    ctx.accounts.token_lottery.ticket_price = price;
    // 记录管理员地址
    ctx.accounts.token_lottery.authority = ctx.accounts.payer.key();
    // 初始化奖池金额为0
    ctx.accounts.token_lottery.lottery_pot_amount = 0;
    // 初始化总票数为0
    ctx.accounts.token_lottery.total_tickets = 0;
    // 初始化随机数账户为默认值
    ctx.accounts.token_lottery.randomness_account = Pubkey::default();
    // 初始化获胜者是否已选出为false
    ctx.accounts.token_lottery.winner_chosen = false;
    Ok(())
}

/// 初始化彩票集合
/// 创建彩票集合的NFT，用于组织所有彩票NFT
///
/// 此函数需要许多账户，因为我们要与Metaplex Token Metadata程序交互来创建NFT：
/// - collection_mint: 集合代币账户
/// - collection_token_account: 集合代币的关联账户
/// - metadata: 集合NFT的元数据账户
/// - master_edition: 集合NFT的主版本账户
/// - token_metadata_program: Metaplex元数据程序
/// - associated_token_program: 关联代币程序
/// - token_program: SPL代币程序
/// - system_program: Solana系统程序
/// - rent: 租金系统变量
pub fn initialize_lottery(ctx: Context<InitializeLottery>) -> Result<()> {
    // 定义签名种子，用于PDA账户的签名
    let signer_seeds: &[&[&[u8]]] = &[&[b"collection_mint".as_ref(), &[ctx.bumps.collection_mint]]];

    msg!("Creating Mint account...");

    // 铸造1个集合代币到集合代币账户
    mint_to(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            MintTo {
                mint: ctx.accounts.collection_mint.to_account_info(),
                to: ctx.accounts.collection_token_account.to_account_info(),
                authority: ctx.accounts.collection_mint.to_account_info(),
            },
            signer_seeds,
        ),
        1,
    )?;
    msg!("Creating Metadata account...");

    // 为集合代币创建元数据账户
    create_metadata_accounts_v3(
        CpiContext::new_with_signer(
            ctx.accounts.token_metadata_program.to_account_info(),
            CreateMetadataAccountsV3 {
                metadata: ctx.accounts.metadata.to_account_info(),
                mint: ctx.accounts.collection_mint.to_account_info(),
                mint_authority: ctx.accounts.collection_mint.to_account_info(),
                payer: ctx.accounts.payer.to_account_info(),
                update_authority: ctx.accounts.collection_mint.to_account_info(),
                system_program: ctx.accounts.system_program.to_account_info(),
                rent: ctx.accounts.rent.to_account_info(),
            },
            signer_seeds,
        ),
        DataV2 {
            name: NAME.to_string(),
            symbol: SYMBOL.to_string(),
            uri: URI.to_string(),
            seller_fee_basis_points: 0,
            creators: Some(vec![Creator {
                address: ctx.accounts.collection_mint.key(),
                verified: false,
                share: 100,
            }]),
            collection: None,
            uses: None,
        },
        true,
        true,
        Some(CollectionDetails::V1 { size: 0 }),
    )?;

    msg!("Creating Master Edition account");
    // 创建主版本账户，使代币成为NFT
    create_master_edition_v3(
        CpiContext::new_with_signer(
            ctx.accounts.token_metadata_program.to_account_info(),
            CreateMasterEditionV3 {
                payer: ctx.accounts.payer.to_account_info(),
                mint: ctx.accounts.collection_mint.to_account_info(),
                edition: ctx.accounts.master_edition.to_account_info(),
                mint_authority: ctx.accounts.collection_mint.to_account_info(),
                update_authority: ctx.accounts.collection_mint.to_account_info(),
                metadata: ctx.accounts.metadata.to_account_info(),
                token_program: ctx.accounts.token_program.to_account_info(),
                system_program: ctx.accounts.system_program.to_account_info(),
                rent: ctx.accounts.rent.to_account_info(),
            },
            &signer_seeds,
        ),
        Some(0),
    )?;

    msg!("verifying collection");
    // 签名并验证集合元数据
    sign_metadata(CpiContext::new_with_signer(
        ctx.accounts.token_metadata_program.to_account_info(),
        SignMetadata {
            creator: ctx.accounts.collection_mint.to_account_info(),
            metadata: ctx.accounts.metadata.to_account_info(),
        },
        signer_seeds,
    ))?;

    Ok(())
}
