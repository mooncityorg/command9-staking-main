use anchor_lang::prelude::*;

use anchor_spl::{
    token::{self, Token, TokenAccount, Transfer }
};
use metaplex_token_metadata::state::Metadata;

pub mod account;
pub mod error;
pub mod constants;

use account::*;
use error::*;
use constants::*;

declare_id!("c7qTRH6XvUCfTUNPjyRotdET7vZ34HSJakwxW3ZdabW");

pub fn vec_to_int(digits: impl IntoIterator<Item = char>) -> Option<u32> {
    const RADIX: u32 = 10;

    digits
        .into_iter()
        .map(|c| c.to_digit(RADIX))
        .try_fold(0, |ans, i| i.map(|i| ans * RADIX + i))
}

#[program]
pub mod basc_staking {
    use super::*;
    pub fn initialize(
        ctx: Context<Initialize>,
        _global_bump: u8,
    ) -> ProgramResult {
        let global_authority = &mut ctx.accounts.global_authority;
        global_authority.super_admin = ctx.accounts.admin.key();
        // Err(ProgramError::from(StakingError::InvalidSuperOwner))
        Ok(())
    }

    pub fn initialize_user_pool(
        ctx: Context<InitializeUserPool>
    ) -> ProgramResult {
        let mut user_pool = ctx.accounts.user_pool.load_init()?;
        user_pool.owner = ctx.accounts.owner.key();
        msg!("Owner: {:?}", user_pool.owner.to_string());
        // Err(ProgramError::from(StakingError::InvalidSuperOwner))
        Ok(())
    }

    pub fn initialize_user_pair_pool(
        ctx: Context<InitializeUserPairPool>
    ) -> ProgramResult {
        let mut user_pool = ctx.accounts.user_pool.load_init()?;
        user_pool.owner = ctx.accounts.owner.key();
        msg!("Pair Owner: {:?}", user_pool.owner.to_string());
        // Err(ProgramError::from(StakingError::InvalidSuperOwner))
        Ok(())
    }

    #[access_control(user(&ctx.accounts.user_pool, &ctx.accounts.owner))]
    pub fn stake_nft_to_pool(
        ctx: Context<StakeNftToPool>,
        _global_bump: u8,
        rank: u64,
    ) -> ProgramResult {
        let mint_metadata = &mut &ctx.accounts.mint_metadata;

        msg!("Metadata Account: {:?}", ctx.accounts.mint_metadata.key());
        let (metadata, _) = Pubkey::find_program_address(
            &[
                metaplex_token_metadata::state::PREFIX.as_bytes(),
                metaplex_token_metadata::id().as_ref(),
                ctx.accounts.nft_mint.key().as_ref(),
            ],
            &metaplex_token_metadata::id(),
        );
        require!(metadata == mint_metadata.key(), StakingError::InvaliedMetadata);

        // verify metadata is legit
        let nft_metadata = Metadata::from_account_info(mint_metadata)?;
        // let parsed_metadata = Data::deserialize(&mut &mint_metadata.data.borrow()[..]).unwrap();
        // msg!("Pared Metadata: {:?}", parsed_metadata.name);
        // let parsed_metadata: Data = try_from_slice_unchecked(&mint_metadata.data.borrow()[..]).unwrap();
        // msg!("NFT Data name: {:?}", nft_metadata.data.name);
        let mut is_ape: u8 = 0;

        if let Some(creators) = nft_metadata.data.creators {
            // metaplex constraints this to max 5, so won't go crazy on compute
            // (empirical testing showed there's practically 0 diff between stopping at 0th and 5th creator)
            let mut valid: u8 = 0;
            let mut collection: Pubkey = Pubkey::default();
            for creator in creators {                
                if (creator.address.to_string() == APE_COLLECTION_ADDRESS || creator.address.to_string() == TIGER_COLLECTION_ADDRESS) && creator.verified == true {
                    if creator.address.to_string() == APE_COLLECTION_ADDRESS {
                        is_ape = 1;
                    } else {
                        is_ape = 0;
                    }
                    valid = 1;
                    collection = creator.address;
                    break;
                }
            }

            require!(valid == 1, StakingError::UnkownOrNotAllowedNFTCollection);
            msg!("Collection= {:?}", collection);
        } else {
            return Err(ProgramError::from(StakingError::MetadataCreatorParseError));
        };

        let mut user_pool = ctx.accounts.user_pool.load_mut()?;
        msg!("Stake Mint: {:?}, Rank: {}", ctx.accounts.nft_mint.key(), rank);

        let timestamp = Clock::get()?.unix_timestamp;
        user_pool.add_nft(ctx.accounts.nft_mint.key(), rank, is_ape as u64, timestamp);

        msg!("Count: {}, Staked Time: {}", user_pool.staked_count, timestamp);
        if is_ape == 1 {
            ctx.accounts.global_authority.total_staked_ape_count += 1;
        } else {
            ctx.accounts.global_authority.total_staked_tiger_count += 1;
        }

        let token_account_info = &mut &ctx.accounts.user_token_account;
        let dest_token_account_info = &mut &ctx.accounts.dest_nft_token_account;
        let token_program = &mut &ctx.accounts.token_program;

        let cpi_accounts = Transfer {
            from: token_account_info.to_account_info().clone(),
            to: dest_token_account_info.to_account_info().clone(),
            authority: ctx.accounts.owner.to_account_info().clone()
        };
        token::transfer(
            CpiContext::new(token_program.clone().to_account_info(), cpi_accounts),
            1
        )?;
        // Err(ProgramError::from(StakingError::InvalidSuperOwner))
        Ok(())
    }

    #[access_control(user(&ctx.accounts.user_pool, &ctx.accounts.owner))]
    pub fn withdraw_nft_from_pool(
        ctx: Context<WithdrawNftFromPool>,
        global_bump: u8,
    ) -> ProgramResult {
        let mut user_pool = ctx.accounts.user_pool.load_mut()?;
        msg!("Staked Mint: {:?}", ctx.accounts.nft_mint.key());

        let timestamp = Clock::get()?.unix_timestamp;
        let is_ape = user_pool.remove_nft(ctx.accounts.nft_mint.key(), timestamp)?;
        msg!("Count: {}, IsApe: {}, Unstaked Time: {}", user_pool.staked_count, is_ape, timestamp);
        if is_ape == 1 {
            ctx.accounts.global_authority.total_staked_ape_count -= 1;
        } else {
            ctx.accounts.global_authority.total_staked_tiger_count -= 1;
        }

        let reward = user_pool.remaining_rewards;
        require!(ctx.accounts.reward_vault.amount >= reward, StakingError::InsufficientRewardVault);
        user_pool.remaining_rewards = 0;

        let token_account_info = &mut &ctx.accounts.user_token_account;
        let dest_token_account_info = &mut &ctx.accounts.dest_nft_token_account;
        let token_program = &mut &ctx.accounts.token_program;
        let seeds = &[GLOBAL_AUTHORITY_SEED.as_bytes(), &[global_bump]];
        let signer = &[&seeds[..]];

        let mut cpi_accounts = Transfer {
            from: dest_token_account_info.to_account_info().clone(),
            to: token_account_info.to_account_info().clone(),
            authority: ctx.accounts.global_authority.to_account_info()
        };
        token::transfer(
            CpiContext::new_with_signer(token_program.clone().to_account_info(), cpi_accounts, signer),
            1
        )?;
        
        cpi_accounts = Transfer {
            from: ctx.accounts.reward_vault.to_account_info(),
            to: ctx.accounts.user_reward_account.to_account_info(),
            authority: ctx.accounts.global_authority.to_account_info()
        };
        token::transfer(
            CpiContext::new_with_signer(token_program.clone().to_account_info(), cpi_accounts, signer),
            reward
        )?;

        // Err(ProgramError::from(StakingError::InvalidSuperOwner))
        Ok(())
    }
    
    #[access_control(user(&ctx.accounts.user_pool, &ctx.accounts.owner))]
    pub fn claim_reward(
        ctx: Context<ClaimReward>,
        global_bump: u8,
        nft_address: Option<Pubkey>,
    ) -> ProgramResult {
        let timestamp = Clock::get()?.unix_timestamp;
    
        let mut user_pool = ctx.accounts.user_pool.load_mut()?;
        let reward: u64;
        if let Some(address) = nft_address {
            reward = user_pool.claim_nft_reward(
                address,
                timestamp,
            )?;
        } else {
            reward = user_pool.claim_reward(
                timestamp
            )?;
        }
        msg!("Reward: {:?}", reward);
        require!(reward > 0, StakingError::InvalidWithdrawTime);
        require!(ctx.accounts.reward_vault.amount >= reward, StakingError::InsufficientRewardVault);

        let seeds = &[GLOBAL_AUTHORITY_SEED.as_bytes(), &[global_bump]];
        let signer = &[&seeds[..]];
        let token_program = ctx.accounts.token_program.to_account_info();
        let cpi_accounts = Transfer {
            from: ctx.accounts.reward_vault.to_account_info(),
            to: ctx.accounts.user_reward_account.to_account_info(),
            authority: ctx.accounts.global_authority.to_account_info()
        };
        token::transfer(
            CpiContext::new_with_signer(token_program.clone(), cpi_accounts, signer),
            reward
        )?;

        // Err(Error::from(StakingError::InvalidSuperOwner))
        Ok(())
    }
    
    #[access_control(user_pair(&ctx.accounts.user_pool, &ctx.accounts.owner))]
    pub fn stake_nft_to_pair_pool(
        ctx: Context<StakeNftToPairPool>,
        _global_bump: u8,
        ape_rank: u64,
        tiger_rank: u64,
    ) -> ProgramResult {
        let ape_mint_metadata = &mut &ctx.accounts.ape_mint_metadata;

        msg!("Metadata Account: {:?}", ctx.accounts.ape_mint_metadata.key());
        let (metadata, _) = Pubkey::find_program_address(
            &[
                metaplex_token_metadata::state::PREFIX.as_bytes(),
                metaplex_token_metadata::id().as_ref(),
                ctx.accounts.ape_nft_mint.key().as_ref(),
            ],
            &metaplex_token_metadata::id(),
        );
        require!(metadata == ape_mint_metadata.key(), StakingError::InvaliedMetadata);

        let tiger_mint_metadata = &mut &ctx.accounts.tiger_mint_metadata;

        msg!("Metadata Account: {:?}", ctx.accounts.tiger_mint_metadata.key());
        let (metadata, _) = Pubkey::find_program_address(
            &[
                metaplex_token_metadata::state::PREFIX.as_bytes(),
                metaplex_token_metadata::id().as_ref(),
                ctx.accounts.tiger_nft_mint.key().as_ref(),
            ],
            &metaplex_token_metadata::id(),
        );
        require!(metadata == tiger_mint_metadata.key(), StakingError::InvaliedMetadata);

        // verify metadata is legit
        let ape_nft_metadata = Metadata::from_account_info(ape_mint_metadata)?;

        if let Some(creators) = ape_nft_metadata.data.creators {
            // metaplex constraints this to max 5, so won't go crazy on compute
            // (empirical testing showed there's practically 0 diff between stopping at 0th and 5th creator)
            let mut valid: u8 = 0;
            let mut collection: Pubkey = Pubkey::default();
            for creator in creators {                
                if creator.address.to_string() == APE_COLLECTION_ADDRESS && creator.verified == true {
                    valid = 1;
                    collection = creator.address;
                    break;
                }
            }

            require!(valid == 1, StakingError::UnkownOrNotAllowedNFTCollection);
            msg!("Collection= {:?}", collection);
        } else {
            return Err(ProgramError::from(StakingError::MetadataCreatorParseError));
        };
        
        let tiger_nft_metadata = Metadata::from_account_info(tiger_mint_metadata)?;

        if let Some(creators) = tiger_nft_metadata.data.creators {
            // metaplex constraints this to max 5, so won't go crazy on compute
            // (empirical testing showed there's practically 0 diff between stopping at 0th and 5th creator)
            let mut valid: u8 = 0;
            let mut collection: Pubkey = Pubkey::default();
            for creator in creators {                
                if creator.address.to_string() == TIGER_COLLECTION_ADDRESS && creator.verified == true {
                    valid = 1;
                    collection = creator.address;
                    break;
                }
            }

            require!(valid == 1, StakingError::UnkownOrNotAllowedNFTCollection);
            msg!("Collection= {:?}", collection);
        } else {
            return Err(ProgramError::from(StakingError::MetadataCreatorParseError));
        };

        let mut char_vec: Vec<char> = ape_nft_metadata.data.name.chars().collect();
        let mut num_array = vec![];
        let mut idx = 0;
        let mut index = 10000;
        while idx < char_vec.len() - 1 {
            if char_vec[idx] == '#' {
                index = idx;
            }
            idx += 1;
            if index != 10000 {
                if u32::from(char_vec[idx]) == 0 {
                    break;
                }
                num_array.push(char_vec[idx]);
            }
        }

        let ape_id: u32 = vec_to_int(num_array).unwrap();

        char_vec = tiger_nft_metadata.data.name.chars().collect();
        num_array = vec![];
        idx = 0;
        index = 10000;
        while idx < char_vec.len() - 1 {
            if char_vec[idx] == '#' {
                index = idx;
            }
            idx += 1;
            if index != 10000 {
                if u32::from(char_vec[idx]) == 0 {
                    break;
                }
                num_array.push(char_vec[idx]);
            }
        }

        let tiger_id: u32 = vec_to_int(num_array).unwrap();

        msg!("Stake Ape Mint: {:?}, Ape Rank: {}, Ape Id: {}", ctx.accounts.ape_nft_mint.key(), ape_rank, ape_id);
        msg!("Stake Tiger Mint: {:?}, Tiger Rank: {}, Tiger Id: {}", ctx.accounts.tiger_nft_mint.key(), tiger_rank, tiger_id);

        let mut user_pool = ctx.accounts.user_pool.load_mut()?;
        let timestamp = Clock::get()?.unix_timestamp;
        user_pool.add_pair(
            ctx.accounts.ape_nft_mint.key(),
            ape_rank,
            ape_id as u64,
            ctx.accounts.tiger_nft_mint.key(),
            tiger_rank,
            tiger_id as u64,
            timestamp
        );

        // msg!("Count: {}, Staked Time: {}", user_pool.staked_pair_count, timestamp);
        ctx.accounts.global_authority.total_staked_ape_count += 1;
        ctx.accounts.global_authority.total_staked_tiger_count += 1;
        ctx.accounts.global_authority.total_staked_pair_count += 1;

        let ape_token_account_info = &mut &ctx.accounts.user_ape_token_account;
        let dest_ape_token_account_info = &mut &ctx.accounts.dest_ape_nft_token_account;
        let token_program = &mut &ctx.accounts.token_program;

        let mut cpi_accounts = Transfer {
            from: ape_token_account_info.to_account_info().clone(),
            to: dest_ape_token_account_info.to_account_info().clone(),
            authority: ctx.accounts.owner.to_account_info().clone()
        };
        token::transfer(
            CpiContext::new(token_program.clone().to_account_info(), cpi_accounts),
            1
        )?;

        let tiger_token_account_info = &mut &ctx.accounts.user_tiger_token_account;
        let dest_tiger_token_account_info = &mut &ctx.accounts.dest_tiger_nft_token_account;

        cpi_accounts = Transfer {
            from: tiger_token_account_info.to_account_info().clone(),
            to: dest_tiger_token_account_info.to_account_info().clone(),
            authority: ctx.accounts.owner.to_account_info().clone()
        };
        token::transfer(
            CpiContext::new(token_program.clone().to_account_info(), cpi_accounts),
            1
        )?;

        // Err(ProgramError::from(StakingError::InvalidSuperOwner))
        Ok(())
    }
    
    #[access_control(user_pair(&ctx.accounts.user_pool, &ctx.accounts.owner))]
    pub fn withdraw_nft_from_pair_pool(
        ctx: Context<WithdrawNftFromPairPool>,
        global_bump: u8,
    ) -> ProgramResult {
        let mut user_pool = ctx.accounts.user_pool.load_mut()?;
        msg!("Staked Ape Mint: {:?}, Staked Tiger Mint: {:?}", ctx.accounts.ape_nft_mint.key(), ctx.accounts.tiger_nft_mint.key());

        let timestamp = Clock::get()?.unix_timestamp;
        user_pool.remove_pair(ctx.accounts.ape_nft_mint.key(), ctx.accounts.tiger_nft_mint.key(), timestamp)?;
        msg!("Count: {}, Unstaked Time: {}", user_pool.staked_pair_count, timestamp);
    
        ctx.accounts.global_authority.total_staked_ape_count -= 1;
        ctx.accounts.global_authority.total_staked_tiger_count -= 1;
        ctx.accounts.global_authority.total_staked_pair_count -= 1;
        
        let reward = user_pool.remaining_rewards;
        require!(ctx.accounts.reward_vault.amount >= reward, StakingError::InsufficientRewardVault);
        user_pool.remaining_rewards = 0;

        let ape_token_account_info = &mut &ctx.accounts.user_ape_token_account;
        let dest_ape_token_account_info = &mut &ctx.accounts.dest_ape_nft_token_account;
        let token_program = &mut &ctx.accounts.token_program;
        let seeds = &[GLOBAL_AUTHORITY_SEED.as_bytes(), &[global_bump]];
        let signer = &[&seeds[..]];

        let mut cpi_accounts = Transfer {
            from: dest_ape_token_account_info.to_account_info().clone(),
            to: ape_token_account_info.to_account_info().clone(),
            authority: ctx.accounts.global_authority.to_account_info()
        };
        token::transfer(
            CpiContext::new_with_signer(token_program.clone().to_account_info(), cpi_accounts, signer),
            1
        )?;

        let tiger_token_account_info = &mut &ctx.accounts.user_tiger_token_account;
        let dest_tiger_token_account_info = &mut &ctx.accounts.dest_tiger_nft_token_account;

        cpi_accounts = Transfer {
            from: dest_tiger_token_account_info.to_account_info().clone(),
            to: tiger_token_account_info.to_account_info().clone(),
            authority: ctx.accounts.global_authority.to_account_info()
        };
        token::transfer(
            CpiContext::new_with_signer(token_program.clone().to_account_info(), cpi_accounts, signer),
            1
        )?;

        cpi_accounts = Transfer {
            from: ctx.accounts.reward_vault.to_account_info(),
            to: ctx.accounts.user_reward_account.to_account_info(),
            authority: ctx.accounts.global_authority.to_account_info()
        };
        token::transfer(
            CpiContext::new_with_signer(token_program.clone().to_account_info(), cpi_accounts, signer),
            reward
        )?;
        // Err(ProgramError::from(StakingError::InvalidSuperOwner))
        Ok(())
    }
    
    #[access_control(user_pair(&ctx.accounts.user_pool, &ctx.accounts.owner))]
    pub fn claim_pair_reward(
        ctx: Context<ClaimPairReward>,
        global_bump: u8,
        ape_nft_address: Option<Pubkey>,
        tiger_nft_address: Option<Pubkey>,
    ) -> ProgramResult {
        let timestamp = Clock::get()?.unix_timestamp;
    
        let mut user_pool = ctx.accounts.user_pool.load_mut()?;
        let reward: u64;
        if let Some(ape_address) = ape_nft_address {
            if let Some(tiger_address) = tiger_nft_address {
                reward = user_pool.claim_pair_reward(ape_address, tiger_address, timestamp)?;
            } else {
                return Err(ProgramError::from(StakingError::InvalidNFTAddress));
            }
        } else {
            if let Some(_tiger_address) = tiger_nft_address {
                return Err(ProgramError::from(StakingError::InvalidNFTAddress));
            } else {
                reward = user_pool.claim_reward(timestamp)?;
            }
        }

        msg!("Reward: {:?} Updated Last Reward Time: {}", reward, user_pool.last_reward_time);
        // msg!("Remaining: {}", user_pool.remaining_rewards);
        require!(reward > 0, StakingError::InvalidWithdrawTime);
        require!(ctx.accounts.reward_vault.amount >= reward, StakingError::InsufficientRewardVault);

        let seeds = &[GLOBAL_AUTHORITY_SEED.as_bytes(), &[global_bump]];
        let signer = &[&seeds[..]];
        let token_program = ctx.accounts.token_program.to_account_info();
        let cpi_accounts = Transfer {
            from: ctx.accounts.reward_vault.to_account_info(),
            to: ctx.accounts.user_reward_account.to_account_info(),
            authority: ctx.accounts.global_authority.to_account_info()
        };
        token::transfer(
            CpiContext::new_with_signer(token_program.clone(), cpi_accounts, signer),
            reward
        )?;

        // Err(ProgramError::from(StakingError::InvalidSuperOwner))
        Ok(())
    }
}

#[derive(Accounts)]
#[instruction(global_bump: u8)]
pub struct Initialize<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,

    #[account(
        init,
        seeds = [GLOBAL_AUTHORITY_SEED.as_ref()],
        bump,
        payer = admin
    )]
    pub global_authority: Account<'info, GlobalPool>,

    #[account(
        mut,
        constraint = reward_vault.mint == REWARD_TOKEN_MINT_PUBKEY.parse::<Pubkey>().unwrap(),
        constraint = reward_vault.owner == global_authority.key(),
        constraint = reward_vault.amount >= MIN_REWARD_DEPOSIT_AMOUNT,
    )]
    pub reward_vault: Account<'info, TokenAccount>,

    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>
}

#[derive(Accounts)]
pub struct InitializeUserPool<'info> {
    #[account(zero)]
    pub user_pool: AccountLoader<'info, UserPool>,

    #[account(mut)]
    pub owner: Signer<'info>,
}

#[derive(Accounts)]
pub struct InitializeUserPairPool<'info> {
    #[account(zero)]
    pub user_pool: AccountLoader<'info, UserPairPool>,

    #[account(mut)]
    pub owner: Signer<'info>,
}

#[derive(Accounts)]
#[instruction(
    global_bump: u8,
)]
pub struct StakeNftToPool<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,

    #[account(mut)]
    pub user_pool: AccountLoader<'info, UserPool>,

    #[account(
        mut,
        seeds = [GLOBAL_AUTHORITY_SEED.as_ref()],
        bump = global_bump,
    )]
    pub global_authority: Box<Account<'info, GlobalPool>>,
    
    #[account(
        mut,
        constraint = user_token_account.mint == *nft_mint.to_account_info().key,
        constraint = user_token_account.owner == *owner.key,
        constraint = user_token_account.amount == 1,
    )]
    pub user_token_account: Account<'info, TokenAccount>,

    #[account(
        mut,
        constraint = dest_nft_token_account.mint == *nft_mint.to_account_info().key,
        constraint = dest_nft_token_account.owner == *global_authority.to_account_info().key,
    )]
    pub dest_nft_token_account: Account<'info, TokenAccount>,
    /// CHECK: This is not dangerous because we don't read or write from this account
    pub nft_mint: AccountInfo<'info>,
    
    #[account(
        mut,
    )]
    /// CHECK: This is not dangerous because we don't read or write from this account
    pub mint_metadata: AccountInfo<'info>,

    pub token_program: Program<'info, Token>,
    /// CHECK: This is not dangerous because we don't read or write from this account
    #[account(constraint = token_metadata_program.key == &metaplex_token_metadata::ID)]
    pub token_metadata_program: AccountInfo<'info>,
}

#[derive(Accounts)]
#[instruction(
    global_bump: u8,
)]
pub struct WithdrawNftFromPool<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,

    #[account(mut)]
    pub user_pool: AccountLoader<'info, UserPool>,

    #[account(
        mut,
        seeds = [GLOBAL_AUTHORITY_SEED.as_ref()],
        bump = global_bump,
    )]
    pub global_authority: Box<Account<'info, GlobalPool>>,
    
    #[account(
        mut,
        constraint = user_token_account.mint == *nft_mint.to_account_info().key,
        constraint = user_token_account.owner == *owner.key,
    )]
    pub user_token_account: Account<'info, TokenAccount>,
    
    #[account(
        mut,
        constraint = dest_nft_token_account.mint == *nft_mint.to_account_info().key,
        constraint = dest_nft_token_account.owner == *global_authority.to_account_info().key,
        constraint = dest_nft_token_account.amount == 1,
    )]
    pub dest_nft_token_account: Account<'info, TokenAccount>,
    /// CHECK: This is not dangerous because we don't read or write from this account
    pub nft_mint: AccountInfo<'info>,
    
    #[account(
        mut,
        constraint = reward_vault.mint == REWARD_TOKEN_MINT_PUBKEY.parse::<Pubkey>().unwrap(),
        constraint = reward_vault.owner == global_authority.key(),
    )]
    pub reward_vault: Box<Account<'info, TokenAccount>>,
    
    #[account(
        mut,
        constraint = user_reward_account.mint == REWARD_TOKEN_MINT_PUBKEY.parse::<Pubkey>().unwrap(),
        constraint = user_reward_account.owner == owner.key(),
    )]
    pub user_reward_account: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
#[instruction(global_bump: u8)]
pub struct ClaimReward<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,

    #[account(mut)]
    pub user_pool: AccountLoader<'info, UserPool>,

    #[account(
        mut,
        seeds = [GLOBAL_AUTHORITY_SEED.as_ref()],
        bump = global_bump,
    )]
    pub global_authority: Account<'info, GlobalPool>,

    #[account(
        mut,
        constraint = reward_vault.mint == REWARD_TOKEN_MINT_PUBKEY.parse::<Pubkey>().unwrap(),
        constraint = reward_vault.owner == global_authority.key(),
    )]
    pub reward_vault: Box<Account<'info, TokenAccount>>,

    #[account(
        mut,
        constraint = user_reward_account.mint == REWARD_TOKEN_MINT_PUBKEY.parse::<Pubkey>().unwrap(),
        constraint = user_reward_account.owner == owner.key(),
    )]
    pub user_reward_account: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
#[instruction(
    global_bump: u8,
)]
pub struct StakeNftToPairPool<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,

    #[account(mut)]
    pub user_pool: AccountLoader<'info, UserPairPool>,

    #[account(
        mut,
        seeds = [GLOBAL_AUTHORITY_SEED.as_ref()],
        bump = global_bump,
    )]
    pub global_authority: Box<Account<'info, GlobalPool>>,
    
    #[account(
        mut,
        constraint = user_ape_token_account.mint == *ape_nft_mint.to_account_info().key,
        constraint = user_ape_token_account.owner == *owner.key,
        constraint = user_ape_token_account.amount == 1,
    )]
    pub user_ape_token_account: Box<Account<'info, TokenAccount>>,

    #[account(
        mut,
        constraint = dest_ape_nft_token_account.mint == *ape_nft_mint.to_account_info().key,
        constraint = dest_ape_nft_token_account.owner == *global_authority.to_account_info().key,
    )]
    pub dest_ape_nft_token_account: Box<Account<'info, TokenAccount>>,
    /// CHECK: This is not dangerous because we don't read or write from this account
    pub ape_nft_mint: AccountInfo<'info>,
    
    #[account(
        mut,
    )]
    /// CHECK: This is not dangerous because we don't read or write from this account
    pub ape_mint_metadata: AccountInfo<'info>,
    
    #[account(
        mut,
        constraint = user_tiger_token_account.mint == *tiger_nft_mint.to_account_info().key,
        constraint = user_tiger_token_account.owner == *owner.key,
        constraint = user_tiger_token_account.amount == 1,
    )]
    pub user_tiger_token_account: Box<Account<'info, TokenAccount>>,

    #[account(
        mut,
        constraint = dest_tiger_nft_token_account.mint == *tiger_nft_mint.to_account_info().key,
        constraint = dest_tiger_nft_token_account.owner == *global_authority.to_account_info().key,
    )]
    pub dest_tiger_nft_token_account: Box<Account<'info, TokenAccount>>,
    /// CHECK: This is not dangerous because we don't read or write from this account
    pub tiger_nft_mint: AccountInfo<'info>,
    
    #[account(
        mut,
    )]
    /// CHECK: This is not dangerous because we don't read or write from this account
    pub tiger_mint_metadata: AccountInfo<'info>,

    pub token_program: Program<'info, Token>,
    
    /// CHECK: This is not dangerous because we don't read or write from this account
    #[account(constraint = token_metadata_program.key == &metaplex_token_metadata::ID)]
    pub token_metadata_program: AccountInfo<'info>,
}

#[derive(Accounts)]
#[instruction(
    global_bump: u8,
)]
pub struct WithdrawNftFromPairPool<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,

    #[account(mut)]
    pub user_pool: AccountLoader<'info, UserPairPool>,

    #[account(
        mut,
        seeds = [GLOBAL_AUTHORITY_SEED.as_ref()],
        bump = global_bump,
    )]
    pub global_authority: Box<Account<'info, GlobalPool>>,
    
    #[account(
        mut,
        constraint = reward_vault.mint == REWARD_TOKEN_MINT_PUBKEY.parse::<Pubkey>().unwrap(),
        constraint = reward_vault.owner == global_authority.key(),
    )]
    pub reward_vault: Box<Account<'info, TokenAccount>>,

    #[account(
        mut,
        constraint = user_ape_token_account.mint == *ape_nft_mint.to_account_info().key,
        constraint = user_ape_token_account.owner == *owner.key,
    )]
    pub user_ape_token_account: Box<Account<'info, TokenAccount>>,
    
    #[account(
        mut,
        constraint = dest_ape_nft_token_account.mint == *ape_nft_mint.to_account_info().key,
        constraint = dest_ape_nft_token_account.owner == *global_authority.to_account_info().key,
        constraint = dest_ape_nft_token_account.amount == 1,
    )]
    pub dest_ape_nft_token_account: Box<Account<'info, TokenAccount>>,
    /// CHECK: This is not dangerous because we don't read or write from this account
    pub ape_nft_mint: AccountInfo<'info>,
    
    #[account(
        mut,
        constraint = user_tiger_token_account.mint == *tiger_nft_mint.to_account_info().key,
        constraint = user_tiger_token_account.owner == *owner.key,
    )]
    pub user_tiger_token_account: Box<Account<'info, TokenAccount>>,
    
    #[account(
        mut,
        constraint = dest_tiger_nft_token_account.mint == *tiger_nft_mint.to_account_info().key,
        constraint = dest_tiger_nft_token_account.owner == *global_authority.to_account_info().key,
        constraint = dest_tiger_nft_token_account.amount == 1,
    )]
    pub dest_tiger_nft_token_account: Box<Account<'info, TokenAccount>>,
    /// CHECK: This is not dangerous because we don't read or write from this account
    pub tiger_nft_mint: AccountInfo<'info>,
    
    #[account(
        mut,
        constraint = user_reward_account.mint == REWARD_TOKEN_MINT_PUBKEY.parse::<Pubkey>().unwrap(),
        constraint = user_reward_account.owner == owner.key(),
    )]
    pub user_reward_account: Box<Account<'info, TokenAccount>>,
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
#[instruction(global_bump: u8)]
pub struct ClaimPairReward<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,

    #[account(mut)]
    pub user_pool: AccountLoader<'info, UserPairPool>,

    #[account(
        mut,
        seeds = [GLOBAL_AUTHORITY_SEED.as_ref()],
        bump = global_bump,
    )]
    pub global_authority: Account<'info, GlobalPool>,

    #[account(
        mut,
        constraint = reward_vault.mint == REWARD_TOKEN_MINT_PUBKEY.parse::<Pubkey>().unwrap(),
        constraint = reward_vault.owner == global_authority.key(),
    )]
    pub reward_vault: Box<Account<'info, TokenAccount>>,

    #[account(
        mut,
        constraint = user_reward_account.mint == REWARD_TOKEN_MINT_PUBKEY.parse::<Pubkey>().unwrap(),
        constraint = user_reward_account.owner == owner.key(),
    )]
    pub user_reward_account: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
}

// Access control modifiers
fn user(pool_loader: &AccountLoader<UserPool>, user: &AccountInfo) -> ProgramResult {
    let user_pool = pool_loader.load()?;
    require!(user_pool.owner == *user.key, StakingError::InvalidUserPool);
    Ok(())
}

// Access control modifiers
fn user_pair(pool_loader: &AccountLoader<UserPairPool>, user: &AccountInfo) -> ProgramResult {
    let user_pair_pool = pool_loader.load()?;
    require!(user_pair_pool.owner == *user.key, StakingError::InvalidUserPool);
    Ok(())
}