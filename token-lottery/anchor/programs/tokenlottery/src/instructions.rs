use anchor_lang::prelude::*;
use anchor_lang::system_program;
use anchor_spl::metadata::{
    create_master_edition_v3, create_metadata_accounts_v3,
    mpl_token_metadata::types::{CollectionDetails, Creator, DataV2},
    set_and_verify_sized_collection_item, sign_metadata, CreateMasterEditionV3,
    CreateMetadataAccountsV3, SetAndVerifySizedCollectionItem, SignMetadata,
};
use anchor_spl::token_interface::{mint_to, MintTo};
use switchboard_on_demand::accounts::RandomnessAccountData;

use crate::{accounts_struct::*, constants::*, state::ErrorCode};

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

/// 提交随机数
/// 管理员调用此函数提交Switchboard提供的随机数
///
/// 需要较少账户，因为我们只是保存一个公钥引用：
/// - token_lottery: 我们的彩票账户
/// - randomness_account: Switchboard随机数账户（外部程序账户）
/// - system_program: 系统程序
pub fn commit_randomness(ctx: Context<CommitRandomness>) -> Result<()> {
    // 获取当前区块链时间
    let clock = Clock::get()?;
    let token_lottery = &mut ctx.accounts.token_lottery;

    // 检查调用者是否为管理员
    if ctx.accounts.payer.key() != token_lottery.authority {
        return Err(ErrorCode::Unauthorized.into());
    }

    // 解析随机数账户数据
    let randomness_data =
        RandomnessAccountData::parse(ctx.accounts.randomness_account.data.borrow()).unwrap();

    // 检查随机数是否对应上一个slot，防止重复使用
    if randomness_data.seed_slot != clock.slot - 1 {
        return Err(ErrorCode::RandomnessAlradeyRevealed.into());
    }

    // 保存随机数账户的公钥
    token_lottery.randomness_account = ctx.accounts.randomness_account.key();

    Ok(())
}

/// 揭示获胜者
/// 使用提交的随机数确定获胜者
///
/// 需要最少的账户，因为我们只读取数据并更新我们的账户：
/// - token_lottery: 我们的彩票账户
/// - randomness_account: Switchboard随机数账户
pub fn reveal_winner(ctx: Context<RevealWinner>) -> Result<()> {
    // 获取当前区块链时间
    let clock = Clock::get()?;
    let token_lottery = &mut ctx.accounts.token_lottery;

    // 检查调用者是否为管理员
    if ctx.accounts.payer.key() != token_lottery.authority {
        return Err(ErrorCode::Unauthorized.into());
    }

    // 检查提供的随机数账户是否与之前提交的一致
    if ctx.accounts.randomness_account.key() != token_lottery.randomness_account {
        return Err(ErrorCode::IncorrectRandomnessAccount.into());
    }

    // 检查彩票销售是否已结束
    if clock.slot < token_lottery.end_time {
        return Err(ErrorCode::LotteryNotCompleted.into());
    }

    // 确保获胜者尚未选出
    require!(!token_lottery.winner_chosen, ErrorCode::WinnerAlreadyChosen);

    // 解析随机数数据
    let randomness_data =
        RandomnessAccountData::parse(ctx.accounts.randomness_account.data.borrow()).unwrap();

    // 获取随机值，如果还未解析完成则返回错误
    let reveal_random_value = randomness_data
        .get_value(&clock)
        .map_err(|_| ErrorCode::RandomnessNotResolved)?;

    // 使用随机数计算获胜者（取随机数第一个字节模总票数）
    let winner = reveal_random_value[0] as u64 % token_lottery.total_tickets;

    // 保存获胜者编号并标记已选出
    token_lottery.winner = winner;
    token_lottery.winner_chosen = true;
    Ok(())
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
