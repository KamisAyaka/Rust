use anchor_lang::{prelude::*, solana_program::native_token::LAMPORTS_PER_SOL};
use pyth_solana_receiver_sdk::price_update::{get_feed_id_from_hex, Price, PriceUpdateV2};

use crate::{
    constant::{FEED_ID, MAX_AGE, PRICE_FEED_DECIMALS_ADJUSTMENT},
    error::CustomError,
    state::{Collateral, Config},
};

pub fn check_health_factor(
    collateral: &Account<Collateral>,
    config: &Account<Config>,
    price_feed: &Account<PriceUpdateV2>,
) -> Result<()> {
    let health_factor = caluclate_health_factor(collateral, config, price_feed)?;
    require!(
        health_factor >= config.min_health_factor,
        CustomError::BelowMinHealthFactor,
    );
    Ok(())
}

pub fn caluclate_health_factor(
    collateral: &Account<Collateral>,
    config: &Account<Config>,
    price_feed: &Account<PriceUpdateV2>,
) -> Result<u64> {
    let collateral_value_in_usd = get_usd_value(&collateral.lamport_balance, price_feed)?;

    let collateral_adjusted_for_liquidation_threshold =
        (collateral_value_in_usd * config.liquidation_threshold) / 100;
    if collateral.amount_minted == 0 {
        msg!("Health Factor Max");
        return Ok(u64::MAX);
    }
    let health_factor = (collateral_adjusted_for_liquidation_threshold) / collateral.amount_minted;
    Ok(health_factor)
}

pub fn get_usd_value(amount_in_lamports: &u64, price_feed: &Account<PriceUpdateV2>) -> Result<u64> {
    let feed_id = get_feed_id_from_hex(FEED_ID)?;

    let price: Price = price_feed.get_price_no_older_than(&Clock::get()?, MAX_AGE, &feed_id)?;
    require!(price.price > 0, CustomError::InvalidPrice);

    let price_in_usd = price.price as u128 * PRICE_FEED_DECIMALS_ADJUSTMENT;

    let amount_in_usd: u128 =
        (*amount_in_lamports as u128 * price_in_usd) / (LAMPORTS_PER_SOL as u128);

    Ok(amount_in_usd as u64)
}

pub fn get_lamports_from_usd(
    amount_in_usd: &u64,
    price_feed: &Account<PriceUpdateV2>,
) -> Result<u64> {
    let feed_id = get_feed_id_from_hex(FEED_ID)?;
    let price = price_feed.get_price_no_older_than(&Clock::get()?, MAX_AGE, &feed_id)?;

    require!(price.price > 0, CustomError::InvalidPrice);

    let price_in_usd: u128 = price.price as u128 * PRICE_FEED_DECIMALS_ADJUSTMENT;

    let amount_in_lamports: u128 =
        (*amount_in_usd as u128 * (LAMPORTS_PER_SOL as u128)) / price_in_usd;
    Ok(amount_in_lamports as u64)
}
