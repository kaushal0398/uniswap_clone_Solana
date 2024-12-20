use anchor_lang::prelude::*;


declare_id!("EYZmnDZqskkz4GcvTU4t5dUPaMwRSgpEX43856gpK1e9");

#[program]
pub mod uniswap_clone {
    use super::*;

    pub fn initialize_pool(
        ctx: Context<InitializePool>,
        initial_token_a: u64,
        initial_token_b: u64,
    ) -> Result<()> {
        let pool = &mut ctx.accounts.pool;
        pool.token_a_reserve = initial_token_a;
        pool.token_b_reserve = initial_token_b;
        pool.total_lp_supply = 0; // No LP tokens initially
        Ok(())
    }

    pub fn add_liquidity(
        ctx: Context<AddLiquidity>,
        token_a_amount: u64,
        token_b_amount: u64,
    ) -> Result<()> {
        let pool = &mut ctx.accounts.pool;

        let lp_tokens = if pool.total_lp_supply == 0 {
            token_a_amount + token_b_amount // Initial LP supply
        } else {
            let lp_a = (token_a_amount as u128 * pool.total_lp_supply as u128)
                / pool.token_a_reserve as u128;
            let lp_b = (token_b_amount as u128 * pool.total_lp_supply as u128)
                / pool.token_b_reserve as u128;
            std::cmp::min(lp_a, lp_b) as u64
        };

        pool.token_a_reserve += token_a_amount;
        pool.token_b_reserve += token_b_amount;
        pool.total_lp_supply += lp_tokens;

        Ok(())
    }

    pub fn swap(
        ctx: Context<Swap>,
        amount_in: u64,
        min_amount_out: u64,
        is_token_a: bool,
    ) -> Result<()> {
        let pool = &mut ctx.accounts.pool;

        let _amount_out = if is_token_a {
            // Token A → Token B
            let numerator = pool.token_b_reserve as u128 * amount_in as u128;
            let denominator = pool.token_a_reserve as u128 + amount_in as u128;
            let amount_out = numerator / denominator;

            require!(
                amount_out >= min_amount_out as u128,
                ErrorCode::SlippageExceeded
            );

            // Update reserves
            pool.token_a_reserve += amount_in;
            pool.token_b_reserve -= amount_out as u64;

            amount_out as u64
        } else {
            // Token B → Token A
            let numerator = pool.token_a_reserve as u128 * amount_in as u128;
            let denominator = pool.token_b_reserve as u128 + amount_in as u128;
            let amount_out = numerator / denominator;

            // slippage
            require!(
                amount_out >= min_amount_out as u128,
                ErrorCode::SlippageExceeded
            );

            // Update reserves according to you
            pool.token_b_reserve += amount_in;
            pool.token_a_reserve -= amount_out as u64;

            amount_out as u64
        };

        Ok(())
    }

    pub fn remove_liquidity(ctx: Context<RemoveLiquidity>, lp_tokens: u64) -> Result<()> {
        let pool = &mut ctx.accounts.pool;

        // Calculate reserves to withdraw
        let token_a_withdraw =
            (lp_tokens as u128 * pool.token_a_reserve as u128) / pool.total_lp_supply as u128;
        let token_b_withdraw =
            (lp_tokens as u128 * pool.token_b_reserve as u128) / pool.total_lp_supply as u128;

        // Update reserves and burn LP tokens
        pool.token_a_reserve -= token_a_withdraw as u64;
        pool.token_b_reserve -= token_b_withdraw as u64;
        pool.total_lp_supply -= lp_tokens;

        Ok(())
    }
}

#[derive(Accounts)]
pub struct InitializePool<'info> {
    #[account(init, payer = user, space = 8 + 8 + 8 + 8)]
    pub pool: Account<'info, Pool>,
    #[account(mut)]
    pub user: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct AddLiquidity<'info> {
    #[account(mut)]
    pub pool: Account<'info, Pool>,
    pub user: Signer<'info>,
}

#[derive(Accounts)]
pub struct Swap<'info> {
    #[account(mut)]
    pub pool: Account<'info, Pool>,
    pub user: Signer<'info>,
}

#[derive(Accounts)]
pub struct RemoveLiquidity<'info> {
    #[account(mut)]
    pub pool: Account<'info, Pool>,
    pub user: Signer<'info>,
}

#[account]
pub struct Pool {
    pub token_a_reserve: u64,
    pub token_b_reserve: u64,
    pub total_lp_supply: u64,
}

#[error_code]
pub enum ErrorCode {
    #[msg("Slippage exceeded")]
    SlippageExceeded,
}
