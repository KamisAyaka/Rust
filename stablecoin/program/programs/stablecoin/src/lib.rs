#![allow(unexpected_cfgs)]
#![allow(deprecated)]
use anchor_lang::prelude::*;
use instructions::*;

pub mod constant;
pub mod error;
pub mod instructions;
pub mod state;

declare_id!("SYqoq2UfwhZdepJoL2sqJZbTGVbp4V8ZAmLpVXNdH47");

#[program]
pub mod stablecoin {
    use super::*;

    pub fn initialize_config(ctx: Context<InitializeConfig>) -> Result<()> {
        instructions::admin::process_initialize_config(ctx)
    }
    pub fn update_config(ctx: Context<UpdateConfig>, min_health_factor: u64) -> Result<()> {
        instructions::admin::process_update_config(ctx, min_health_factor)
    }

    pub fn deposit_and_mint_token(
        ctx: Context<DepositAndMintToken>,
        amount_collateral: u64,
        amount_mint: u64,
    ) -> Result<()> {
        instructions::deposit::process_deposit_and_mint_token(ctx, amount_collateral, amount_mint)
    }

    pub fn redeem_collateral_and_burn_token(
        ctx: Context<RedeemCollateralAndBurnToken>,
        amount_collateral: u64,
        amount_to_burn: u64,
    ) -> Result<()> {
        instructions::withdraw::process_redeem_collateral_and_burn_token(
            ctx,
            amount_collateral,
            amount_to_burn,
        )
    }

    pub fn liquidate(ctx: Context<Liquidate>, amount_to_burn: u64) -> Result<()> {
        instructions::withdraw::process_liquidate(ctx, amount_to_burn)
    }
}
