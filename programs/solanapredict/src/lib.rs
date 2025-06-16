use anchor_lang::prelude::*;

declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");

#[program]
pub mod solanapredict {
    use super::*;

    pub fn create_market(
        ctx: Context<CreateMarket>,
        question: String,
        description: String,
        end_time: i64,
    ) -> Result<()> {
        let market = &mut ctx.accounts.market;
        let clock = Clock::get()?;

        require!(end_time > clock.unix_timestamp, ErrorCode::InvalidEndTime);
        require!(question.len() <= 200, ErrorCode::QuestionTooLong);
        require!(description.len() <= 500, ErrorCode::DescriptionTooLong);

        market.creator = ctx.accounts.creator.key();
        market.question = question;
        market.description = description;
        market.end_time = end_time;
        market.yes_pool = 0;
        market.no_pool = 0;
        market.total_bets = 0;
        market.is_resolved = false;
        market.winning_option = None;
        market.created_at = clock.unix_timestamp;

        Ok(())
    }

    pub fn place_bet(
        ctx: Context<PlaceBet>,
        amount: u64,
        option: BetOption,
    ) -> Result<()> {
        let market = &mut ctx.accounts.market;
        let bet = &mut ctx.accounts.bet;
        let clock = Clock::get()?;

        require!(!market.is_resolved, ErrorCode::MarketResolved);
        require!(clock.unix_timestamp < market.end_time, ErrorCode::MarketEnded);
        require!(amount > 0, ErrorCode::InvalidAmount);

        // Transfer SOL from user to market
        let ix = anchor_lang::solana_program::system_instruction::transfer(
            &ctx.accounts.user.key(),
            &market.key(),
            amount,
        );
        anchor_lang::solana_program::program::invoke(
            &ix,
            &[
                ctx.accounts.user.to_account_info(),
                market.to_account_info(),
            ],
        )?;

        // Update market pools
        match option {
            BetOption::Yes => market.yes_pool += amount,
            BetOption::No => market.no_pool += amount,
        }

        // Create bet record
        bet.market = market.key();
        bet.user = ctx.accounts.user.key();
        bet.amount = amount;
        bet.option = option;
        bet.timestamp = clock.unix_timestamp;
        bet.claimed = false;

        market.total_bets += 1;

        Ok(())
    }

    pub fn resolve_market(
        ctx: Context<ResolveMarket>,
        winning_option: BetOption,
    ) -> Result<()> {
        let market = &mut ctx.accounts.market;
        let clock = Clock::get()?;

        require!(!market.is_resolved, ErrorCode::AlreadyResolved);
        require!(clock.unix_timestamp >= market.end_time, ErrorCode::MarketNotEnded);
        require!(
            ctx.accounts.resolver.key() == market.creator,
            ErrorCode::UnauthorizedResolver
        );

        market.is_resolved = true;
        market.winning_option = Some(winning_option);
        market.resolved_at = Some(clock.unix_timestamp);

        Ok(())
    }

    pub fn claim_winnings(ctx: Context<ClaimWinnings>) -> Result<()> {
        let market = &ctx.accounts.market;
        let bet = &mut ctx.accounts.bet;

        require!(market.is_resolved, ErrorCode::MarketNotResolved);
        require!(!bet.claimed, ErrorCode::AlreadyClaimed);
        require!(bet.user == ctx.accounts.user.key(), ErrorCode::UnauthorizedClaim);

        let winning_option = market.winning_option.unwrap();
        require!(bet.option == winning_option, ErrorCode::LosingBet);

        let total_pool = market.yes_pool + market.no_pool;
        let winning_pool = match winning_option {
            BetOption::Yes => market.yes_pool,
            BetOption::No => market.no_pool,
        };

        // Calculate winnings: (user_bet / winning_pool) * total_pool
        let winnings = (bet.amount as u128 * total_pool as u128) / winning_pool as u128;

        // Transfer winnings to user
        **market.to_account_info().try_borrow_mut_lamports()? -= winnings as u64;
        **ctx.accounts.user.to_account_info().try_borrow_mut_lamports()? += winnings as u64;

        bet.claimed = true;

        Ok(())
    }
}

#[derive(Accounts)]
#[instruction(question: String, description: String)]
pub struct CreateMarket<'info> {
    #[account(
        init,
        payer = creator,
        space = 8 + Market::INIT_SPACE,
        seeds = [b"market", creator.key().as_ref(), question.as_bytes()],
        bump
    )]
    pub market: Account<'info, Market>,
    #[account(mut)]
    pub creator: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct PlaceBet<'info> {
    #[account(mut)]
    pub market: Account<'info, Market>,
    #[account(
        init,
        payer = user,
        space = 8 + Bet::INIT_SPACE,
        seeds = [b"bet", market.key().as_ref(), user.key().as_ref()],
        bump
    )]
    pub bet: Account<'info, Bet>,
    #[account(mut)]
    pub user: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct ResolveMarket<'info> {
    #[account(mut)]
    pub market: Account<'info, Market>,
    pub resolver: Signer<'info>,
}

#[derive(Accounts)]
pub struct ClaimWinnings<'info> {
    #[account(mut)]
    pub market: Account<'info, Market>,
    #[account(mut)]
    pub bet: Account<'info, Bet>,
    #[account(mut)]
    pub user: Signer<'info>,
}

#[account]
#[derive(InitSpace)]
pub struct Market {
    pub creator: Pubkey,
    #[max_len(200)]
    pub question: String,
    #[max_len(500)]
    pub description: String,
    pub end_time: i64,
    pub yes_pool: u64,
    pub no_pool: u64,
    pub total_bets: u64,
    pub is_resolved: bool,
    pub winning_option: Option<BetOption>,
    pub created_at: i64,
    pub resolved_at: Option<i64>,
}

#[account]
#[derive(InitSpace)]
pub struct Bet {
    pub market: Pubkey,
    pub user: Pubkey,
    pub amount: u64,
    pub option: BetOption,
    pub timestamp: i64,
    pub claimed: bool,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq, InitSpace)]
pub enum BetOption {
    Yes,
    No,
}

#[error_code]
pub enum ErrorCode {
    #[msg("Invalid end time")]
    InvalidEndTime,
    #[msg("Question too long")]
    QuestionTooLong,
    #[msg("Description too long")]
    DescriptionTooLong,
    #[msg("Market already resolved")]
    MarketResolved,
    #[msg("Market has ended")]
    MarketEnded,
    #[msg("Invalid amount")]
    InvalidAmount,
    #[msg("Market already resolved")]
    AlreadyResolved,
    #[msg("Market has not ended yet")]
    MarketNotEnded,
    #[msg("Unauthorized resolver")]
    UnauthorizedResolver,
    #[msg("Market not resolved")]
    MarketNotResolved,
    #[msg("Winnings already claimed")]
    AlreadyClaimed,
    #[msg("Unauthorized claim")]
    UnauthorizedClaim,
    #[msg("This is a losing bet")]
    LosingBet,
}