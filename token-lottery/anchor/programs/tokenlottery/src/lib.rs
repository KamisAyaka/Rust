#![allow(unexpected_cfgs)]

use anchor_lang::prelude::*;
use instructions::*;

declare_id!("38eCiZu5v6EjeZyn575E9uGZEqUstrV6qVvemnpeq3tp");

pub mod constants;
pub mod error;
pub mod instructions;
pub mod state;

#[program]
pub mod tokenlottery {
    use super::*;

    pub fn initialize_config(
        ctx: Context<Initialize>,
        start: u64,
        end: u64,
        price: u64,
    ) -> Result<()> {
        instructions::initialize_config(ctx, start, end, price)
    }

    pub fn initialize_lottery(ctx: Context<InitializeLottery>) -> Result<()> {
        instructions::initialize_lottery(ctx)
    }

    pub fn buy_ticket(ctx: Context<BuyTicket>) -> Result<()> {
        instructions::buy_ticket(ctx)
    }

    pub fn commit_randomness(ctx: Context<CommitRandomness>) -> Result<()> {
        instructions::commit_randomness(ctx)
    }

    pub fn reveal_winner(ctx: Context<RevealWinner>) -> Result<()> {
        instructions::reveal_winner(ctx)
    }

    pub fn claim_winnings(ctx: Context<ClaimWinnings>) -> Result<()> {
        instructions::claim_winnings(ctx)
    }
}
