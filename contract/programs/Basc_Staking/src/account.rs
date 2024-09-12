use anchor_lang::prelude::*;

use crate::constants::*;
use crate::error::*;

#[account]
#[derive(Default)]
pub struct GlobalPool {
    pub super_admin: Pubkey,            // 32
    pub total_staked_ape_count: u64,    // 8
    pub total_staked_tiger_count: u64,  // 8
    pub total_staked_pair_count: u64,  // 8
}

#[zero_copy]
#[derive(Default, PartialEq)]
#[repr(packed)]
pub struct StakedData {
    pub mint: Pubkey,             // 32
    pub last_claimed_time: i64,   // 8
    pub rank: u64,                // 8
    pub is_ape: u64,                // 8
}

#[account(zero_copy)]
pub struct UserPool {
    // 8 + 5656
    pub owner: Pubkey,                                 // 32
    pub staked_count: u64,                             // 8
    pub staked_nfts: [StakedData; STAKE_MAX_COUNT],    // 56 * 100
    pub last_reward_time: i64,                         // 8
    pub remaining_rewards: u64,                        // 8
}

impl Default for UserPool {
    #[inline]
    fn default() -> UserPool {
        UserPool {
            owner: Pubkey::default(),
            staked_count: 0,
            staked_nfts: [
                StakedData {
                    ..Default::default()
                }; STAKE_MAX_COUNT
            ],
            last_reward_time: 0,
            remaining_rewards: 0,
        }
    }
}

impl UserPool {
    pub fn add_nft(
        &mut self,
        nft_pubkey: Pubkey,
        rank: u64,
        is_ape: u64,
        now: i64,
    ) {
        let idx = self.staked_count as usize;
        self.staked_nfts[idx].mint = nft_pubkey;
        self.staked_nfts[idx].rank = rank;
        self.staked_nfts[idx].is_ape = is_ape;
        self.staked_nfts[idx].last_claimed_time = now;
        self.staked_count += 1;
    }

    pub fn remove_nft(
        &mut self,
        nft_pubkey: Pubkey,
        now: i64,
    ) -> Result<u64> {
        let mut withdrawn: u8 = 0;
        let mut index: usize = 0;
        // Find NFT in pool
        for i in 0..self.staked_count {
            let idx = i as usize;
            if self.staked_nfts[idx].mint.eq(&nft_pubkey) {
                index = idx;
                withdrawn = 1;
                break;
            }
        }
        require!(withdrawn == 1, StakingError::InvalidNFTAddress);

        let is_ape: u64 = self.staked_nfts[index].is_ape;
        // Calculate withdrawing NFT's reward
        let reward_amount: u64;
        let mut last_reward_time: i64 = self.last_reward_time;
        
        if last_reward_time < self.staked_nfts[index].last_claimed_time {
            last_reward_time = self.staked_nfts[index].last_claimed_time;
        }
        
        if self.staked_nfts[index].is_ape == 1 {
            if self.staked_nfts[index].rank < 13 {
                reward_amount = SUPER_APE_REWARD;
            } else if self.staked_nfts[index].rank < 1000 {
                reward_amount = HIGH_APE_REWARD;
            } else {
                reward_amount = NORMAL_APE_REWARD;
            }
        } else {
            if self.staked_nfts[index].rank < 5013 {
                reward_amount = SUPER_TIGER_REWARD;
            } else if self.staked_nfts[index].rank < 6000 {
                reward_amount = HIGH_TIGER_REWARD;
            } else {
                reward_amount = NORMAL_TIGER_REWARD;
            }
        }

        let reward = (((now - last_reward_time) / EPOCH) as u64) * reward_amount as u64;
        self.remaining_rewards = reward;

        // Remove NFT from pool
        let last_idx: usize = (self.staked_count - 1) as usize;
        if index != last_idx {
            self.staked_nfts[index] = self.staked_nfts[last_idx];
        }
        self.staked_count -= 1;

        msg!("Reward: {:?} ", reward);
        Ok(is_ape)
    }
    
    pub fn claim_reward(&mut self, now: i64) -> Result<u64> {
        let mut total_reward: u64 = 0;
        msg!("Now: {:?} Last_Reward_Time: {}", now, self.last_reward_time);
        for i in 0..self.staked_count {
            let index = i as usize;
 
            let mut last_reward_time = self.last_reward_time;
            if last_reward_time < self.staked_nfts[index].last_claimed_time {
                last_reward_time = self.staked_nfts[index].last_claimed_time;
            }

            let reward_amount: u64;

            if self.staked_nfts[index].is_ape == 1 {
                if self.staked_nfts[index].rank < 13 {
                    reward_amount = SUPER_APE_REWARD;
                } else if self.staked_nfts[index].rank < 1000 {
                    reward_amount = HIGH_APE_REWARD;
                } else {
                    reward_amount = NORMAL_APE_REWARD;
                }
            } else {
                if self.staked_nfts[index].rank < 5013 {
                    reward_amount = SUPER_TIGER_REWARD;
                } else if self.staked_nfts[index].rank < 6000 {
                    reward_amount = HIGH_TIGER_REWARD;
                } else {
                    reward_amount = NORMAL_TIGER_REWARD;
                }
            }
            
            let reward: u64 = (((now - last_reward_time) / EPOCH) as u64) * reward_amount as u64;

            total_reward += reward;
        }
        // total_reward += self.remaining_rewards;
        self.last_reward_time = now;
        // msg!("Remaining: {}", self.remaining_rewards);
        self.remaining_rewards = 0;
        Ok(total_reward)
    }

    pub fn claim_nft_reward(
        &mut self,
        nft_pubkey: Pubkey,
        now: i64,
    ) -> Result<u64> {
        let mut withdrawn: u8 = 0;
        let mut index: usize = 0;
        // Find NFT in pool
        for i in 0..self.staked_count {
            let idx = i as usize;
            if self.staked_nfts[idx].mint.eq(&nft_pubkey) {
                index = idx;
                withdrawn = 1;
                break;
            }
        }
        require!(withdrawn == 1, StakingError::InvalidNFTAddress);
        
        // Calculate withdrawing NFT's reward
        let reward_amount: u64;
        let mut last_reward_time: i64 = self.last_reward_time;
        
        if last_reward_time < self.staked_nfts[index].last_claimed_time {
            last_reward_time = self.staked_nfts[index].last_claimed_time;
        }
        
        if self.staked_nfts[index].is_ape == 1 {
            if self.staked_nfts[index].rank < 13 {
                reward_amount = SUPER_APE_REWARD;
            } else if self.staked_nfts[index].rank < 1000 {
                reward_amount = HIGH_APE_REWARD;
            } else {
                reward_amount = NORMAL_APE_REWARD;
            }
        } else {
            if self.staked_nfts[index].rank < 5013 {
                reward_amount = SUPER_TIGER_REWARD;
            } else if self.staked_nfts[index].rank < 6000 {
                reward_amount = HIGH_TIGER_REWARD;
            } else {
                reward_amount = NORMAL_TIGER_REWARD;
            }
        }

        let reward = (((now - last_reward_time) / EPOCH) as u64) * reward_amount as u64;
        msg!("Last Claimed Time: {:?}", last_reward_time);
        self.staked_nfts[index].last_claimed_time = now;
        Ok(reward)
    }
}

#[zero_copy]
#[derive(Default, PartialEq)]
#[repr(packed)]
pub struct PairStakedData {
    pub ape_mint: Pubkey,               // 32
    pub ape_rank: u64,                  // 8
    pub ape_id: u64,                    // 8
    pub tiger_mint: Pubkey,             // 32
    pub tiger_rank: u64,                // 8
    pub tiger_id: u64,                  // 8
    pub last_claimed_time: i64,         // 8
}

#[account(zero_copy)]
pub struct UserPairPool {
    // 8 + 6712
    pub owner: Pubkey,                                           // 32
    pub staked_pair_count: u64,                                  // 8
    pub staked_pairs: [PairStakedData; PAIR_STAKE_MAX_COUNT],    // 104 * 64
    pub last_reward_time: i64,                                   // 8
    pub remaining_rewards: u64,                                  // 8
}

impl Default for UserPairPool {
    #[inline]
    fn default() -> UserPairPool {
        UserPairPool {
            owner: Pubkey::default(),
            staked_pair_count: 0,
            staked_pairs: [
                PairStakedData {
                    ..Default::default()
                }; PAIR_STAKE_MAX_COUNT
            ],
            last_reward_time: 0,
            remaining_rewards: 0,
        }
    }
}

impl UserPairPool {
    pub fn add_pair(
        &mut self,
        ape_pubkey: Pubkey,
        ape_rank: u64,
        ape_id: u64,
        tiger_pubkey: Pubkey,
        tiger_rank: u64,
        tiger_id: u64,
        now: i64,
    ) {
        let idx = self.staked_pair_count as usize;
        self.staked_pairs[idx].ape_mint = ape_pubkey;
        self.staked_pairs[idx].ape_rank = ape_rank;
        self.staked_pairs[idx].ape_id = ape_id;
        self.staked_pairs[idx].tiger_mint = tiger_pubkey;
        self.staked_pairs[idx].tiger_rank = tiger_rank;
        self.staked_pairs[idx].tiger_id = tiger_id;
        self.staked_pairs[idx].last_claimed_time = now;
        self.staked_pair_count += 1;
    }
    
    pub fn remove_pair(
        &mut self,
        ape_pubkey: Pubkey,
        tiger_pubkey: Pubkey,
        now: i64,
    ) -> Result<u64> {
        let mut withdrawn: u8 = 0;
        let mut index: usize = 0;
        let mut factor: u64 = 100;

        // Find NFT in pool
        for i in 0..self.staked_pair_count {
            let idx = i as usize;
            if self.staked_pairs[idx].ape_mint.eq(&ape_pubkey) {
                require!(self.staked_pairs[idx].tiger_mint.eq(&tiger_pubkey), StakingError::InvalidNFTAddress);
                index = idx;
                withdrawn = 1;
                let ape_id = self.staked_pairs[idx].ape_id;
                if ape_id == self.staked_pairs[idx].tiger_id {
                    msg!("Matched Pair: {}", ape_id);
                    factor = MATCHING_FACTOR;
                }
                break;
            }
        }
        require!(withdrawn == 1, StakingError::InvalidNFTAddress);

        // Calculate withdrawing NFT's reward
        let mut last_reward_time: i64 = self.last_reward_time;
        let ape_reward_amount: u64;
        let tiger_reward_amount: u64;

        if last_reward_time < self.staked_pairs[index].last_claimed_time {
            last_reward_time = self.staked_pairs[index].last_claimed_time;
        }

        if self.staked_pairs[index].ape_rank < 13 {
            ape_reward_amount = SUPER_APE_REWARD;
        } else if self.staked_pairs[index].ape_rank < 1000 {
            ape_reward_amount = HIGH_APE_REWARD;
        } else {
            ape_reward_amount = NORMAL_APE_REWARD;
        }
        
        if self.staked_pairs[index].tiger_rank < 5013 {
            tiger_reward_amount = SUPER_TIGER_REWARD;
        } else if self.staked_pairs[index].tiger_rank < 6000 {
            tiger_reward_amount = HIGH_TIGER_REWARD;
        } else {
            tiger_reward_amount = NORMAL_TIGER_REWARD;
        }

        let reward = (((now - last_reward_time) / EPOCH) as u64) * (ape_reward_amount + tiger_reward_amount) * factor / 100 as u64;
        self.remaining_rewards = reward;

        // Remove NFT from pool
        let last_idx: usize = (self.staked_pair_count - 1) as usize;
        if index != last_idx {
            self.staked_pairs[index] = self.staked_pairs[last_idx];
            // self.staked_pairs[index].is_legendary = self.staked_pairs[last_idx].is_legendary;
            // self.staked_pairs[index].staked_time = self.staked_pairs[last_idx].staked_time;
        }
        self.staked_pair_count -= 1;
        Ok(reward)
    }
    
    pub fn claim_reward(&mut self, now: i64) -> Result<u64> {
        let mut total_reward: u64 = 0;
        msg!("Now: {:?} Last_Reward_Time: {}", now, self.last_reward_time);
        for i in 0..self.staked_pair_count {
            let index = i as usize;
 
            let mut last_reward_time = self.last_reward_time;
            if last_reward_time < self.staked_pairs[index].last_claimed_time {
                last_reward_time = self.staked_pairs[index].last_claimed_time;
            }

            let mut factor: u64 = 100;
            let ape_reward_amount: u64;
            let tiger_reward_amount: u64;
            if self.staked_pairs[index].ape_id == self.staked_pairs[index].tiger_id {
                factor = MATCHING_FACTOR;
            }
    
            if self.staked_pairs[index].ape_rank < 13 {
                ape_reward_amount = SUPER_APE_REWARD;
            } else if self.staked_pairs[index].ape_rank < 1000 {
                ape_reward_amount = HIGH_APE_REWARD;
            } else {
                ape_reward_amount = NORMAL_APE_REWARD;
            }
            
            if self.staked_pairs[index].tiger_rank < 5013 {
                tiger_reward_amount = SUPER_TIGER_REWARD;
            } else if self.staked_pairs[index].tiger_rank < 6000 {
                tiger_reward_amount = HIGH_TIGER_REWARD;
            } else {
                tiger_reward_amount = NORMAL_TIGER_REWARD;
            }
    
            let reward = (((now - last_reward_time) / EPOCH) as u64) * (ape_reward_amount + tiger_reward_amount) * factor / 100 as u64;
            total_reward += reward;
        }
        // total_reward += self.remaining_rewards;
        self.last_reward_time = now;
        self.remaining_rewards = 0;
        Ok(total_reward)
    }

    pub fn claim_pair_reward(&mut self, ape_pubkey: Pubkey, tiger_pubkey: Pubkey, now: i64) -> Result<u64> {
        let mut withdrawn: u8 = 0;
        let mut index: usize = 0;
        let mut factor: u64 = 100;

        // Find NFT in pool
        for i in 0..self.staked_pair_count {
            let idx = i as usize;
            if self.staked_pairs[idx].ape_mint.eq(&ape_pubkey) {
                require!(self.staked_pairs[idx].tiger_mint.eq(&tiger_pubkey), StakingError::InvalidNFTAddress);
                index = idx;
                withdrawn = 1;
                let ape_id = self.staked_pairs[idx].ape_id;
                if ape_id == self.staked_pairs[idx].tiger_id {
                    msg!("Matched Pair: {}", ape_id);
                    factor = MATCHING_FACTOR;
                }
                break;
            }
        }
        require!(withdrawn == 1, StakingError::InvalidNFTAddress);

        // Calculate withdrawing NFT's reward
        let mut last_reward_time: i64 = self.last_reward_time;
        let ape_reward_amount: u64;
        let tiger_reward_amount: u64;

        if last_reward_time < self.staked_pairs[index].last_claimed_time {
            last_reward_time = self.staked_pairs[index].last_claimed_time;
        }

        if self.staked_pairs[index].ape_rank < 13 {
            ape_reward_amount = SUPER_APE_REWARD;
        } else if self.staked_pairs[index].ape_rank < 1000 {
            ape_reward_amount = HIGH_APE_REWARD;
        } else {
            ape_reward_amount = NORMAL_APE_REWARD;
        }
        
        if self.staked_pairs[index].tiger_rank < 5013 {
            tiger_reward_amount = SUPER_TIGER_REWARD;
        } else if self.staked_pairs[index].tiger_rank < 6000 {
            tiger_reward_amount = HIGH_TIGER_REWARD;
        } else {
            tiger_reward_amount = NORMAL_TIGER_REWARD;
        }

        let reward = (((now - last_reward_time) / EPOCH) as u64) * (ape_reward_amount + tiger_reward_amount) * factor / 100 as u64;
        self.staked_pairs[index].last_claimed_time = now;

        Ok(reward)
    }
}