use anchor_lang::prelude::*;

#[error_code]
pub enum CustomError {
    #[msg("Invalid Price")]
    InvalidPrice,
    #[msg("Below Min Health Factor")]
    BelowMinHealthFactor,
    #[msg("Health Factor Too High")]
    HealthFactorTooHigh,
}
