#![allow(unexpected_cfgs)]
#![allow(deprecated)]
pub mod constants;
pub mod error;
pub mod instructions;
pub mod state;

use anchor_lang::prelude::*;

pub use constants::*;
pub use instructions::*;
pub use state::*;

declare_id!("GkAaGQj9ETzMtYsYgdkfJxXUDTZQ7ZEQQcujci4T27y9");

#[program]
pub mod swap {
    use super::*;

    pub fn make_offer(
        ctx: Context<MakeOffer>,
        id: u64,
        token_a_offered_amount: u64,
        token_b_wanted_amount: u64,
    ) -> Result<()> {
        instructions::make_offer::send_offered_tokens_to_vault(&ctx, token_a_offered_amount)?;
        instructions::make_offer::save_offer(ctx, id, token_b_wanted_amount)
    }

    pub fn take_offer(context: Context<TakeOffer>) -> Result<()> {
        // 检查报价是否已被取消
        require!(!context.accounts.offer.is_cancelled, crate::error::ErrorCode::OfferAlreadyCancelled);
        
        instructions::take_offer::send_wanted_tokens_to_maker(&context)?;
        instructions::take_offer::withdraw_and_close_vault(&context)
    }

    pub fn cancel_offer(ctx: Context<CancelOffer>, _offer_id: u64) -> Result<()> {
        instructions::cancel_offer::cancel_offer(ctx)
    }
}
