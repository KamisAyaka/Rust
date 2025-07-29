use anchor_lang::prelude::*;

/// 报价信息结构体
/// 
/// 存储链上报价的核心信息，包括交易对、数量、创建者等
#[account]
#[derive(InitSpace)]
pub struct Offer {
    /// 报价ID
    pub offer_id: u64,
    /// 报价创建者公钥
    pub maker: Pubkey,
    /// 代币A的Mint地址（创建者提供）
    pub token_mint_a: Pubkey,
    /// 代币B的Mint地址（创建者想要）
    pub token_mint_b: Pubkey,
    /// 创建者想要的代币B数量
    pub token_b_wanted_amount: u64,
    /// PDA账户的bump种子
    pub bump: u8,
    /// 报价是否已被取消的标志
    pub is_cancelled: bool,
}