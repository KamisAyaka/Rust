use crate::error::ErrorCode;
use crate::state::TokenLottery;
use anchor_lang::prelude::*;
use switchboard_on_demand::RandomnessAccountData;

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
