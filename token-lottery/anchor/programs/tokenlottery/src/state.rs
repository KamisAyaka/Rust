use anchor_lang::prelude::*;

/// 彩票账户数据结构
#[account]
#[derive(InitSpace)]
pub struct TokenLottery {
    pub bump: u8,                   // PDA的bump种子
    pub winner: u64,                // 获胜者编号
    pub winner_chosen: bool,        // 是否已选出获胜者
    pub start_time: u64,            // 开始时间（slot）
    pub end_time: u64,              // 结束时间（slot）
    pub lottery_pot_amount: u64,    // 奖池金额
    pub ticket_price: u64,          // 彩票价格
    pub total_tickets: u64,         // 总票数
    pub authority: Pubkey,          // 管理员公钥
    pub randomness_account: Pubkey, // 随机数账户公钥
}

/// 错误码枚举
#[error_code]
pub enum ErrorCode {
    #[msg("Lottery is not open")] // 彩票未开放
    LotteryNotOpen,
    #[msg("Not authorized")] // 未授权
    Unauthorized,
    #[msg("Randomness already revealed")] // 随机数已揭示
    RandomnessAlradeyRevealed,
    #[msg("Incorrect randomness account")] // 错误的随机数账户
    IncorrectRandomnessAccount,
    #[msg("Lottery is not Completed")] // 彩票未完成
    LotteryNotCompleted,
    #[msg("Winner already chosen")] // 获胜者已选出
    WinnerAlreadyChosen,
    #[msg("Randomness not resolved")] // 随机数未解析
    RandomnessNotResolved,
    #[msg("Winner not chosen")] // 获胜者未选出
    WinnerNotChosen,
    #[msg("Collection not verified")] // 集合未验证
    CollectionNotVerified,
    #[msg("Incorrect ticket")] // 错误的彩票
    IncorrectTicket,
    #[msg("No ticket")] // 没有彩票
    NoTicket,
}
