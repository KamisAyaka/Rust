use anchor_lang::prelude::*;

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
