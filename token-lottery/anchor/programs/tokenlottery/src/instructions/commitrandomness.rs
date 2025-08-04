use crate::error::ErrorCode;
use crate::state::TokenLottery;
use anchor_lang::prelude::*;
use switchboard_on_demand::RandomnessAccountData;

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
