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
