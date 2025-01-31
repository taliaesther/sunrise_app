use crate::state::State;
use crate::utils::marinade;
use crate::utils::seeds::{BSOL_ACCOUNT, MSOL_ACCOUNT};
use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token, TokenAccount};
use marinade_cpi::program::MarinadeFinance;
use marinade_cpi::State as MarinadeState;
use std::ops::Deref;

#[derive(Accounts, Clone)]
pub struct ExtractToTreasury<'info> {
    #[account(
    has_one = treasury,
    has_one = marinade_state,
    has_one = blaze_state,
    )]
    pub state: Box<Account<'info, State>>,

    #[account(mut)]
    pub marinade_state: Box<Account<'info, MarinadeState>>,

    /// CHECK: Checked
    pub blaze_state: AccountInfo<'info>,

    #[account(mut)]
    pub msol_mint: Box<Account<'info, Mint>>,

    #[account()]
    pub gsol_mint: Box<Account<'info, Mint>>,

    pub bsol_mint: Box<Account<'info, Mint>>,

    #[account()]
    pub liq_pool_mint: Box<Account<'info, Mint>>,

    #[account(mut)]
    /// CHECK: Checked in marinade program
    pub liq_pool_sol_leg_pda: AccountInfo<'info>,

    #[account(mut)]
    /// CHECK: Checked in marinade program
    pub liq_pool_msol_leg: Box<Account<'info, TokenAccount>>,

    #[account(
    mut,
    token::mint = liq_pool_mint,
    // use the same authority PDA for this and the msol token account
    token::authority = get_msol_from_authority
    )]
    pub liq_pool_token_account: Box<Account<'info, TokenAccount>>,

    #[account(mut)]
    /// CHECK: Checked in marinade program
    pub treasury_msol_account: Box<Account<'info, TokenAccount>>,

    #[account(
    mut,
    token::mint = msol_mint,
    token::authority = get_msol_from_authority,
    )]
    pub get_msol_from: Box<Account<'info, TokenAccount>>,

    #[account(
    seeds = [state.key().as_ref(), MSOL_ACCOUNT],
    bump = state.msol_authority_bump
    )]
    pub get_msol_from_authority: SystemAccount<'info>, // sunrise-stake PDA

    #[account(
    mut,
    token::mint = bsol_mint,
    token::authority = get_bsol_from_authority,
    )]
    pub get_bsol_from: Box<Account<'info, TokenAccount>>,

    #[account(
    seeds = [state.key().as_ref(), BSOL_ACCOUNT],
    bump = state.bsol_authority_bump
    )]
    pub get_bsol_from_authority: SystemAccount<'info>, // sunrise-stake PDA

    #[account(mut)]
    /// CHECK: Matches state.treasury
    pub treasury: SystemAccount<'info>, // sunrise-stake treasury

    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub marinade_program: Program<'info, MarinadeFinance>,
}

pub fn extract_to_treasury_handler(ctx: Context<ExtractToTreasury>) -> Result<()> {
    // TODO at present, this withdraws all msol yield. In future, we should be able to choose how much to withdraw
    let extractable_yield = marinade::calculate_extractable_yield(
        ctx.accounts,
        &ctx.accounts.get_msol_from,
        &ctx.accounts.get_bsol_from,
        &ctx.accounts.gsol_mint,
    )?;

    let extractable_yield_msol =
        marinade::calc_msol_from_lamports(ctx.accounts.marinade_state.as_ref(), extractable_yield)?;

    // TODO later change to use "slow unstake" rather than incur liq pool fees

    msg!("Withdrawing {} msol to treasury", extractable_yield);
    let accounts = ctx.accounts.deref().into();
    marinade::unstake(&accounts, extractable_yield_msol)?;

    Ok(())
}
