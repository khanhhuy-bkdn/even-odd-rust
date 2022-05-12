use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::{UnorderedSet, UnorderedMap};
use near_sdk::{env, near_bindgen, AccountId, Promise};

near_sdk::setup_alloc!();

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize)]
pub struct EvenOdd {
    owner_id: AccountId,
    total_bet_amount: u128,
    total_bet_amount_per_roll: u128,
    roll_id: u128,
    players_array: UnorderedSet<AccountId>,
    players: UnorderedMap<AccountId, PlayerMetadata>
}

impl Default for EvenOdd {
    fn default() -> Self {
        env::panic(b"The contract should be initialized before usage")
    }
}

#[derive(BorshDeserialize, BorshSerialize, Default, Debug)]
pub struct PlayerMetadata {
    bet_amount: u128,
    player: AccountId,
    is_even: bool,
}

#[near_bindgen]
impl EvenOdd {
    #[init]
    pub fn new(owner_id: AccountId) -> Self {
        assert!(!env::state_exists(), "The contract is already initialized");
        assert!(
            env::is_valid_account_id(owner_id.as_bytes()),
            "Owner's account ID is invalid."
        );
        Self {
            owner_id,
            total_bet_amount: 0,
            total_bet_amount_per_roll: 0,
            roll_id: 1,
            players_array: UnorderedSet::new(b"players_array".to_vec()),
            players: UnorderedMap::new(b"players".to_vec()),
        }
    }

    pub(crate) fn assert_owner(&self) {
        assert_eq!(
            env::predecessor_account_id(),
            self.owner_id,
            "Can only be called by the owner"
        );
    }

    #[payable]
    pub fn bet(&mut self, is_even: bool) {
        println!("account_balance {}", env::account_balance());
        println!("account_locked_balance {}", env::account_locked_balance());
        println!("account_signer {}", env::signer_account_id());
        let account = env::predecessor_account_id();
        let amount = env::attached_deposit();
        assert_ne!(
            amount,
            0,
            "minimum amount needed to play the game."
        );
        assert!(
            self.is_already_bet(account.clone()),
            "Already bet"
        );
        
        self.players.insert(&account, &PlayerMetadata { bet_amount: amount, player: account.clone(), is_even: is_even});

        self.players_array.insert(&account);

        self.total_bet_amount += amount;
        self.total_bet_amount_per_roll += amount;

        let log_message = format!("Bet at {}, account: {}, amount: {}, is_even: {}", self.roll_id, account, amount, is_even);
        env::log(log_message.as_bytes());
    }

    pub fn roll_dice(&mut self) {
        self.assert_owner();

        let dice_number_1: u8 = self.generate_random_number();
        let dice_number_2: u8 = self.generate_random_number();

        let is_even: bool = (dice_number_1 + dice_number_2) % 2 == 0;

        let log_message = format!("DiceRolled at {}, dice number 1: {}, dice number 2: {}, is_even: {}", self.roll_id, dice_number_1, dice_number_2, is_even);
        env::log(log_message.as_bytes());

        for account_id in self.players_array.iter() {
            let data = self.players.get(&account_id).unwrap();
            if data.is_even == is_even {
                let amount = self.players.get(&account_id).unwrap_or_default().bet_amount;
                Promise::new(account_id).transfer(amount * 2);
            }
        }
        self.reset_board();
        self.roll_id += 1;
    }

    pub fn reset_board(&mut self) {
        self.assert_owner();

        for account_id in self.players_array.iter() {
            self.players.remove(&account_id);
        }
        self.players_array.clear();
        self.total_bet_amount_per_roll = 0;
    }

    pub fn withdraw(&mut self, amount: near_sdk::json_types::U128) -> Promise {
        Promise::new(self.owner_id.clone()).transfer(amount.0)
    }

    fn generate_random_number(&mut self) -> u8 {
        let rand: u8 = *env::random_seed().get(0).unwrap();
        rand
    }

    // get function
    pub fn get_owner(&self) -> AccountId {
        self.owner_id.clone()
    }

    pub fn get_player(&self, account_id: AccountId) -> PlayerMetadata {
        match self.players.get(&account_id) {
            Some(data) => data,
            None => PlayerMetadata::default(),
        }
    }

    pub fn get_roll_id(&self) -> u128 {
        self.roll_id
    }

    pub fn get_total_bet_amount(&self) -> u128 {
        self.total_bet_amount
    }

    pub fn get_total_bet_amount_per_roll(&self) -> u128 {
        self.total_bet_amount_per_roll
    }

    pub fn is_already_bet(&self, account: AccountId) -> bool {
        match self.players.get(&account) {
            Some(data) => if data.bet_amount > 0 { return true; } else { return false;},
            None => true
        }
    }
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests {
    use near_sdk::MockedBlockchain;
    use near_sdk::json_types::ValidAccountId;
    use near_sdk::test_utils::{accounts, VMContextBuilder};
    use near_sdk::testing_env;

    use super::*;

    const MINT_STORAGE_COST: u128 = 5870000000000000000000;

    fn get_context(predecessor_account_id: ValidAccountId) -> VMContextBuilder {
        let mut builder = VMContextBuilder::new();
        builder
            .current_account_id(accounts(0))
            .signer_account_id(predecessor_account_id.clone())
            .predecessor_account_id(predecessor_account_id);
        builder
    }

    #[test]
    fn test_new() {
        let mut context = get_context(accounts(1));
        testing_env!(context.build());
        let contract = EvenOdd::new(accounts(0).into());
        testing_env!(context.is_view(true).build());
        assert_eq!(contract.get_owner(), accounts(0).to_string());
    }

    #[test]
    fn test_bet() {
        let mut context = get_context(accounts(1));
        testing_env!(context.build());
        let mut contract = EvenOdd::new(accounts(0).into());

        testing_env!(context
            .storage_usage(env::storage_usage())
            .attached_deposit(MINT_STORAGE_COST)
            .predecessor_account_id(accounts(0))
            .build());
        contract.bet(true);

        let mut result = contract.get_player(accounts(0).to_string());
        assert_eq!(result.bet_amount, MINT_STORAGE_COST);
        assert_eq!(result.is_even, true);
        assert_eq!(result.player, accounts(0).to_string());

        contract.bet(false);
        result = contract.get_player(accounts(0).to_string());
        assert_eq!(result.bet_amount, MINT_STORAGE_COST);
        assert_eq!(result.is_even, false);
        assert_eq!(result.player, accounts(0).to_string());

        assert_eq!(contract.get_total_bet_amount_per_roll(), MINT_STORAGE_COST * 2);
        assert_eq!(contract.get_total_bet_amount(), MINT_STORAGE_COST * 2);

        contract.reset_board();
    }

    #[test]
    fn test_reset_board() {
        let mut context = get_context(accounts(1));
        testing_env!(context.build());
        let mut contract = EvenOdd::new(accounts(0).into());

        testing_env!(context
            .storage_usage(env::storage_usage())
            .attached_deposit(MINT_STORAGE_COST)
            .predecessor_account_id(accounts(1))
            .build());
        println!("account_balance before {}", env::account_balance());
        contract.bet(true);

        context = get_context(accounts(2));
        testing_env!(context
            .storage_usage(env::storage_usage())
            .attached_deposit(MINT_STORAGE_COST)
            .predecessor_account_id(accounts(2))
            .build());

        contract.bet(false);
        println!("account_balance after {}", env::account_balance());
        println!("owner contract {}", contract.get_owner());

        println!("{:?}", contract.get_player(accounts(1).to_string()));
        println!("{:?}", contract.get_player(accounts(2).to_string()));

        context = get_context(accounts(0));
        testing_env!(context
            .storage_usage(env::storage_usage())
            .attached_deposit(MINT_STORAGE_COST)
            .predecessor_account_id(accounts(0))
            .build());

        contract.reset_board();
    }
}