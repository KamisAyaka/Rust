use crate::state::TokenLottery;
use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    metadata::{Metadata, MetadataAccount},
    token_interface::{Mint, TokenAccount, TokenInterface},
};

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

/// 定义提交随机数的账户结构体
/// 这是最简单的账户结构之一，只需要与外部Switchboard程序交互。
#[derive(Accounts)]
pub struct CommitRandomness<'info> {
    #[account(mut)]
    pub payer: Signer<'info>, // 提交随机数的管理员账户

    // 彩票账户
    #[account(
        mut,
        seeds = [b"token_lottery".as_ref()],
        bump = token_lottery.bump,
    )]
    pub token_lottery: Account<'info, TokenLottery>,

    /// CHECK: This account is checked by the Switchboard smart contract
    pub randomness_account: UncheckedAccount<'info>, // Switchboard随机数账户

    pub system_program: Program<'info, System>, // 系统程序
}

/// 定义揭示获胜者的账户结构体
/// 只需要读取我们自己的账户和外部随机数账户。
#[derive(Accounts)]
pub struct RevealWinner<'info> {
    #[account(mut)]
    pub payer: Signer<'info>, // 揭示获胜者的管理员账户

    // 彩票账户
    #[account(
        mut,
        seeds = [b"token_lottery".as_ref()],
        bump = token_lottery.bump,
    )]
    pub token_lottery: Account<'info, TokenLottery>,

    /// CHECK: This account is checked by the Switchboard smart contract
    pub randomness_account: UncheckedAccount<'info>, // Switchboard随机数账户
}

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
