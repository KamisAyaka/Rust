use anchor_lang::prelude::*;

/// 彩票NFT名称前缀
#[constant]
pub const NAME: &str = "Token Lottery Ticket #";

/// 彩票代币符号  
#[constant]
pub const SYMBOL: &str = "TLT";

/// 元数据URI
#[constant]
pub const URI: &str = "https://raw.githubusercontent.com/solana-developers/developer-bootcamp-2024/refs/heads/main/project-9-token-lottery/metadata.json";
