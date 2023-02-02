use anchor_lang::prelude::*;
use anchor_lang::solana_program::clock::Clock;

pub const MAX_STAKERS: usize = 2000;

#[account(zero_copy)]
pub struct Vault {
    pub authority: Pubkey,

    pub stake_token_mint: Pubkey,

    pub reward_pool_amount: u64,

    pub total_staked_amount: u64,

    pub daily_payout_amount: u64,

    pub last_updated_time: u64,

    pub users: [User; MAX_STAKERS],

    pub total_user_count: u16,

    pub bump: u8,
}

impl Default for Vault {
    fn default() -> Vault {
        Vault {
            authority: Pubkey::default(),
            stake_token_mint: Pubkey::default(),
            reward_pool_amount: 0,
            total_staked_amount: 0,
            daily_payout_amount: 0,
            last_updated_time: 0,
            users: [User::default(); MAX_STAKERS],
            total_user_count: 0,
            bump: 0,
        }
    }
}

impl Vault {
    pub const LEN: usize = std::mem::size_of::<Vault>();

    pub fn update(&mut self) {
        let now: u64 = Clock::get().unwrap().unix_timestamp.try_into().unwrap();
        if self.last_updated_time == 0 {
            self.last_updated_time = now;
            return;
        }

        let staked_seconds = now.checked_sub(self.last_updated_time).unwrap();
        for i in 0..self.total_user_count as usize {
            let earned_amount = self
                .daily_payout_amount
                .checked_mul(staked_seconds)
                .unwrap()
                .checked_div(86400)
                .unwrap()
                .checked_div(self.total_staked_amount)
                .unwrap()
                .checked_mul(self.users[i].staked_amount)
                .unwrap();                
            self.users[i].earned_amount = self.users[i]
                .earned_amount
                .checked_add(earned_amount)
                .unwrap();
        }
        self.last_updated_time = now;
    }

    pub fn stake(&mut self, key: Pubkey, amount: u64) {
        self.update();

        for i in 0..self.total_user_count as usize {
            if self.users[i].key == key {
                self.users[i].staked_amount =
                    self.users[i].staked_amount.checked_add(amount).unwrap();
                self.total_staked_amount = self.total_staked_amount.checked_add(amount).unwrap();
                return;
            }
        }
        self.users[self.total_user_count as usize] = User {
            key: key,
            staked_amount: amount,
            earned_amount: 0,
        };
        self.total_user_count = self.total_user_count.checked_add(1).unwrap();
        self.total_staked_amount = self.total_staked_amount.checked_add(amount).unwrap();
    }

    pub fn unstake(&mut self, key: Pubkey, amount: u64) {
        self.update();

        for i in 0..self.total_user_count as usize {
            if self.users[i].key == key {
                self.users[i].staked_amount =
                    self.users[i].staked_amount.checked_sub(amount).unwrap();
                if self.users[i].staked_amount == 0 {
                    self.users[i] = self.users[self.total_user_count as usize - 1];
                    self.users[self.total_user_count as usize - 1] = User::default();
                    self.total_user_count = self.total_user_count.checked_sub(1).unwrap();
                    self.total_staked_amount =
                        self.total_staked_amount.checked_sub(amount).unwrap();
                }
                return;
            }
        }
    }

    pub fn claim(&mut self, key: Pubkey) -> u64 {
        self.update();

        for i in 0..self.total_user_count as usize {
            if self.users[i].key == key {
                let earned_amount = self.users[i].earned_amount;
                self.users[i].earned_amount = 0;
                self.reward_pool_amount =
                    self.reward_pool_amount.checked_sub(earned_amount).unwrap();
                return earned_amount;
            }
        }
        return 0;
    }
}

#[zero_copy]
#[derive(Debug, PartialEq)]
pub struct User {
    pub key: Pubkey,
    pub staked_amount: u64,
    pub earned_amount: u64,
}

impl Default for User {
    fn default() -> User {
        User {
            key: Pubkey::default(),
            staked_amount: 0,
            earned_amount: 0,
        }
    }
}
