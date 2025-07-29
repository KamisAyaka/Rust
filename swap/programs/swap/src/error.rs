use anchor_lang::prelude::*;

#[error_code]
pub enum ErrorCode {
    /// 通用自定义错误
    #[msg("Custom error message")]
    CustomError,

    /// 调用者不是报价创建者
    #[msg("Only the offer maker can cancel the offer")]
    NotMaker,

    /// 报价已经被取消
    #[msg("Offer has already been cancelled")]
    OfferAlreadyCancelled,

    /// 代币Mint地址不匹配
    #[msg("Token mint does not match the offer")]
    WrongTokenMint,
}
