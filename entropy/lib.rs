#![cfg_attr(not(feature = "std"), no_std)]

use ink_lang as ink;

#[ink::contract]
mod entropy {
    use core::fmt;

    use ink_env as env;

    use ink_prelude::{
        format,
        string::String
    };

    use ink_storage::{
        collections::HashMap as StorageHashMap,
        lazy::Lazy,
    };

    /// Defines the storage of your contract.
    /// Add new fields to the below struct in order
    /// to add new static storage fields to your contract.
    #[ink(storage)]
    pub struct Entropy {
        name: String,
        symbol: String,
        decimals: u32,

        /// Additional params for use if contract level transaction fees ever became necessary
        basis_points_rate: u128, // e.g: '5' means 0.0005 rate
        maximum_fee: u128,  // e.g: '50_000_000' means maximal 50 ENT fee per trasaction

        owner: AccountId,

        /// Total token supply.
        total_supply: Lazy<Balance>,

        /// Mapping from owner to number of owned token.
        balances: StorageHashMap<AccountId, Balance>,

        /// Mapping of the token amount which an account is allowed to withdraw
        /// from another account.
        allowances: StorageHashMap<(AccountId, AccountId), Balance>,

        /// Mapping of whether an account is private
        accounts_private: StorageHashMap<AccountId, bool>,

        /// Mapping of whether an account is blacklisted
        accounts_blacklisted: StorageHashMap<AccountId, bool>
    }

    
    /// Event emitted when params are set.
    #[ink(event)]
    pub struct Params {
        #[ink(topic)]
        basis_points_rate: u128,
        #[ink(topic)]
        maximum_fee: u128
    }

    /// Event emitted when a token transfer occurs.
    #[ink(event)]
    pub struct Transfer {
        #[ink(topic)]
        from: Option<AccountId>,
        #[ink(topic)]
        to: Option<AccountId>,
        #[ink(topic)]
        value: Balance,
    }

    /// Event emitted when an approval occurs that `spender` is allowed to withdraw
    /// up to the amount of `value` tokens from `owner`.
    #[ink(event)]
    pub struct Approval {
        #[ink(topic)]
        owner: AccountId,
        #[ink(topic)]
        spender: AccountId,
        #[ink(topic)]
        value: Balance,
    }

    /// Event emitted when new tokens are issued
    #[ink(event)]
    pub struct Issue {
        #[ink(topic)]
        amount: Balance
    }
    
    /// Event emitted when new tokens are redeemed
    #[ink(event)]
    pub struct Redeem {
        #[ink(topic)]
        amount: Balance
    }

    /// Event emitted when an account's privacy is updated
    #[ink(event)]
    pub struct Privacy {
        #[ink(topic)]
        account: AccountId,
        #[ink(topic)]
        private: bool
    }

    /// Event emitted when an account is blacklisted
    #[ink(event)]
    pub struct AddedBlackList {
        #[ink(topic)]
        account: AccountId
    }

    /// Event emitted when an account is removed from blacklist
    #[ink(event)]
    pub struct RemovedBlackList {
        #[ink(topic)]
        account: AccountId
    }

    /// Event emitted when a blacklisted account's fund is destroyed
    #[ink(event)]
    pub struct DestroyedBlackFunds {
        #[ink(topic)]
        account: AccountId,
        #[ink(topic)]
        funds: Balance
    }

    /// Event emitted when error occurs
    #[ink(event)]
    pub struct TransactionFailed {
        #[ink(topic)]
        error: String
    }

    /// Entropy error types.
    #[derive(Debug, PartialEq, Eq, scale::Encode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum Error {
        /// Returned if not privileged.
        PermissionDenied,
        /// Returned if not enough balance to fulfill a request is available.
        InsufficientBalance,
        /// Returned if not enough allowance to fulfill a request is available.
        InsufficientAllowance,
        /// Returned if trying to destropy funds of an account which is not blacklisted
        AccountNotBlackListed
    }

    impl fmt::Display for Error {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            match *self {
                Self::PermissionDenied => write!(f, "PermissionDenied"),
                Self::InsufficientBalance => write!(f, "InsufficientBalance"),
                Self::InsufficientAllowance => write!(f, "InsufficientAllowance"),
                Self::AccountNotBlackListed => write!(f, "AccountNotBlackListed")
            }
        }
    }

    /// Entropy result type.
    pub type Result<T> = core::result::Result<T, Error>;

    impl Entropy {

        /// Creates a new Entropy contract with the specified initial supply, name, symbol and decimals.
        #[ink(constructor)]
        pub fn construct(initial_supply: Balance, name: String, symbol: String, decimals: u32) -> Self {
            env::debug_println(&format!("Entropy: Construct with initial_supply: 0x{:x}, name: {}, symbol: {}, decimals: 0x{:x}", initial_supply, &name, &symbol, decimals));

            let caller = Self::env().caller();
            let mut balances = StorageHashMap::new();
            balances.insert(caller, initial_supply);
            let instance = Self {
                total_supply: Lazy::new(initial_supply),
                name: name.clone(),
                symbol: symbol.clone(),
                basis_points_rate: 0,
                maximum_fee: 0,
                owner: caller,
                decimals,
                balances,
                allowances: StorageHashMap::new(),
                accounts_private: StorageHashMap::new(),
                accounts_blacklisted: StorageHashMap::new()
            };
            Self::env().emit_event(Transfer {
                from: None,
                to: Some(caller),
                value: initial_supply,
            });
            instance
        }

        /// Creates a new Entropy contract with the specified initial supply and default name, symbol and decimals.
        #[ink(constructor)]
        pub fn new(initial_supply: Balance) -> Self {
            Entropy::construct(initial_supply, "Entropy Coin".into(), "ENT".into(), 6)
        }

        /// Creates a new Entropy contract with default initial supply, name, symbol and decimals.
        #[ink(constructor)]
        pub fn default() -> Self {
            Entropy::construct(1_000_000_000_000, "Entropy Coin".into(), "ENT".into(), 6)
        }

        /// Returns the token name.
        #[ink(message)]
        pub fn name(&self) -> String {
            self.name.clone()
        }

        /// Returns the token symbol.
        #[ink(message)]
        pub fn symbol(&self) -> String {
            self.symbol.clone()
        }

        /// Returns the token decimals.
        #[ink(message)]
        pub fn decimals(&self) -> u32 {
            self.decimals
        }

        /// Returns contract level transaction fee basic points rate (*/10000)
        #[ink(message)]
        pub fn basis_points_rate(&self) -> u128 {
            self.basis_points_rate
        }

        /// Returns contract level maximum fee per transaction
        #[ink(message)]
        pub fn maximum_fee(&self) -> u128 {
            self.maximum_fee
        }

        /// Set contract level transaction fee params
        #[ink(message)]
        pub fn set_params(&mut self, new_basic_points: u128, new_max_fee: u128) -> Result<()> {
            let caller = self.env().caller();
            if caller != self.owner {
                self.env().emit_event(TransactionFailed {
                    error: format!("{:?}", Error::PermissionDenied)
                });
                return Err(Error::PermissionDenied);
            }

            self.basis_points_rate = if new_basic_points > 20 { 20 } else { new_basic_points };
            self.maximum_fee = if new_max_fee > 50_000_000 { 50_000_000 } else { new_max_fee };

            self.env().emit_event(Params {
                basis_points_rate: self.basis_points_rate,
                maximum_fee: self.maximum_fee
            });

            Ok(())
        }

        /// Returns the contract owner.
        #[ink(message)]
        pub fn owner(&self) -> AccountId {
            self.owner
        }

        /// Returns the total token supply.
        #[ink(message)]
        pub fn total_supply(&self) -> Balance {
            *self.total_supply
        }

        /// Returns the account balance for the specified `owner`.
        ///
        /// Returns `0` if the account is non-existent.
        #[ink(message)]
        pub fn balance_of(&self, owner: AccountId) -> Balance {
            self.balances.get(&owner).copied().unwrap_or(0)
        }

        /// Returns the amount which `spender` is still allowed to withdraw from `owner`.
        ///
        /// Returns `0` if no allowance has been set `0`.
        #[ink(message)]
        pub fn allowance(&self, owner: AccountId, spender: AccountId) -> Balance {
            self.allowances.get(&(owner, spender)).copied().unwrap_or(0)
        }

        /// Transfer ownership to another account
        #[ink(message)]
        pub fn transfer_ownership(&mut self, new_owner: AccountId) -> Result<()> {
            let caller = self.env().caller();
            if caller != self.owner {
                self.env().emit_event(TransactionFailed {
                    error: format!("{:?}", Error::PermissionDenied)
                });
                return Err(Error::PermissionDenied);
            }

            if new_owner != AccountId::from([0x0; 32]) {
                self.owner = new_owner.clone();
            }
            Ok(())
        }

        /// Transfers `value` amount of tokens from the caller's account to account `to`.
        ///
        /// On success a `Transfer` event is emitted.
        ///
        /// # Errors
        ///
        /// Returns `InsufficientBalance` error if there are not enough tokens on
        /// the caller's account balance.
        #[ink(message)]
        pub fn transfer(&mut self, to: AccountId, value: Balance) -> Result<()> {
            let from = self.env().caller();
            self.transfer_from_to(from, to, value)
        }

        /// Allows `spender` to withdraw from the caller's account multiple times, up to
        /// the `value` amount.
        ///
        /// If this function is called again it overwrites the current allowance with `value`.
        ///
        /// An `Approval` event is emitted.
        #[ink(message)]
        pub fn approve(&mut self, spender: AccountId, value: Balance) -> Result<()> {
            let owner = self.env().caller();
            self.allowances.insert((owner, spender), value);
            self.env().emit_event(Approval {
                owner,
                spender,
                value,
            });
            Ok(())
        }

        /// Transfers `value` tokens on the behalf of `from` to the account `to`.
        ///
        /// This can be used to allow a contract to transfer tokens on ones behalf and/or
        /// to charge fees in sub-currencies, for example.
        ///
        /// On success a `Transfer` event is emitted.
        ///
        /// # Errors
        ///
        /// Returns `InsufficientAllowance` error if there are not enough tokens allowed
        /// for the caller to withdraw from `from`.
        ///
        /// Returns `InsufficientBalance` error if there are not enough tokens on
        /// the the account balance of `from`.
        #[ink(message)]
        pub fn transfer_from(
            &mut self,
            from: AccountId,
            to: AccountId,
            value: Balance,
        ) -> Result<()> {
            env::debug_println(&format!("Entropy: Trying to transfer 0x{:x} tokens from {:?} to {:?}", value, from, to));

            let caller = self.env().caller();
            let allowance = self.allowance(from, caller);
            if allowance < value {
                self.env().emit_event(TransactionFailed {
                    error: format!("{:?}", Error::InsufficientAllowance)
                });
                return Err(Error::InsufficientAllowance)
            }
            self.transfer_from_to(from, to, value)?;
            self.allowances.insert((from, caller), allowance - value);
            Ok(())
        }

        /// Transfers `value` amount of tokens from the caller's account to account `to`.
        ///
        /// On success a `Transfer` event is emitted.
        ///
        /// # Errors
        ///
        /// Returns `InsufficientBalance` error if there are not enough tokens on
        /// the caller's account balance.
        fn transfer_from_to(
            &mut self,
            from: AccountId,
            to: AccountId,
            value: Balance,
        ) -> Result<()> {
            env::debug_println(&format!("Entropy: Transferring 0x{:x} tokens from {:?} to {:?}", value, from, to));

            let from_balance = self.balance_of(from);
            if from_balance < value {
                self.env().emit_event(TransactionFailed {
                    error: format!("{:?}", Error::InsufficientBalance)
                });
                return Err(Error::InsufficientBalance)
            }

            let mut fee = 0;
            if self.basis_points_rate > 0 {
                // let init_fee = value.checked_mul(Balance::from(self.basis_points_rate)).unwrap_or(Balance::from(0u128)).checked_div(Balance::from(10000u128)).unwrap_or(Balance::from(0u128));
                let init_fee = value * self.basis_points_rate / 10000;
                fee = if init_fee > self.maximum_fee { self.maximum_fee } else { init_fee };
            }
            let send_value = value - fee;

            self.balances.insert(from, from_balance - value);
            let to_balance = self.balance_of(to);
            self.balances.insert(to, to_balance + send_value);

            if fee > 0 {
                let owner_balance = self.balance_of(self.owner);
                self.balances.insert(self.owner, owner_balance + fee);
                self.env().emit_event(Transfer {
                    from: Some(from),
                    to: Some(self.owner),
                    value: fee
                });
            }

            self.env().emit_event(Transfer {
                from: Some(from),
                to: Some(to),
                value: send_value,
            });
            Ok(())
        }

        /// Issues `value` amount of tokens to contract owner's account. Only contract owner is allowed to call this function.
        /// 
        /// On success a `Issue` event is emitted.
        /// 
        /// # Errors
        /// 
        /// Returns `PermissionDenied` error if caller is not the owner.
        #[ink(message)]
        pub fn issue(&mut self, value: Balance) -> Result<()> {
            env::debug_println(&format!("Entropy: Issuing 0x{:x} tokens to owner account", value));

            let caller = self.env().caller();
            if caller != self.owner {
                self.env().emit_event(TransactionFailed {
                    error: format!("{:?}", Error::PermissionDenied)
                });
                return Err(Error::PermissionDenied);
            }

            let balance = self.balance_of(self.owner);
            self.balances.insert(self.owner, balance + value);

            let total_supply = &mut self.total_supply;
            let current_supply = Lazy::<Balance>::get(total_supply);
            let new_supply = current_supply + value;
            Lazy::<Balance>::set(total_supply, new_supply);

            self.env().emit_event(Issue {
                amount: value
            });

            Ok(())
        }

        /// Redeem `value` amount of tokens from contract owner's account. Only contract owner is allowed to call this function.
        /// 
        /// On success a `Redeem` event is emitted.
        /// 
        /// # Errors
        /// 
        /// Returns `PermissionDenied` error if caller is not the owner.
        /// Returns `InsufficientBalance` error if owner's balance is insufficient.
        #[ink(message)]
        pub fn redeem(&mut self, value: Balance) -> Result<()> {
            env::debug_println(&format!("Entropy: Redeeming 0x{:x} tokens from owner account", value));

            let caller = self.env().caller();
            if caller != self.owner {
                self.env().emit_event(TransactionFailed {
                    error: format!("{:?}", Error::PermissionDenied)
                });
                return Err(Error::PermissionDenied);
            }

            let balance = self.balance_of(self.owner);
            if balance < value {
                self.env().emit_event(TransactionFailed {
                    error: format!("{:?}", Error::InsufficientBalance)
                });
                return Err(Error::InsufficientBalance);
            }

            self.balances.insert(self.owner, balance - value);

            let total_supply = &mut self.total_supply;
            let current_supply = Lazy::<Balance>::get(total_supply);
            let new_supply = current_supply - value;
            Lazy::<Balance>::set(total_supply, new_supply);

            self.env().emit_event(Redeem {
                amount: value
            });

            Ok(())
        }

        /// Set whether an account is private or not
        /// 
        /// On success a `Privacy` event is emitted.
        /// 
        /// # Errors
        /// 
        /// Returns `PermissionDenied` error if caller is not the owner.
        #[ink(message)]
        pub fn set_account_private(&mut self, account: AccountId, private: bool) -> Result<()> {
            let caller = self.env().caller();
            if caller != self.owner {
                self.env().emit_event(TransactionFailed {
                    error: format!("{:?}", Error::PermissionDenied)
                });
                return Err(Error::PermissionDenied);
            }

            self.accounts_private.insert(account, private);

            self.env().emit_event(Privacy {
                account,
                private
            });

            Ok(())
        }

        /// Returns whether an account is private
        #[ink(message)]
        pub fn is_account_private(&self, account: AccountId) -> bool {
            self.accounts_private.get(&account).copied().unwrap_or(false)
        }

        /// Returns whether an account is blacklisted
        #[ink(message)]
        pub fn is_account_blacklisted(&self, account: AccountId) -> bool {
            self.accounts_blacklisted.get(&account).copied().unwrap_or(false)
        }

        /// Add an account to blacklist
        /// 
        /// On success an `AddedBlackList` event is emitted.
        /// 
        /// # Errors
        /// 
        /// Returns `PermissionDenied` error if caller is not the owner.
        #[ink(message)]
        pub fn add_account_to_blacklist(&mut self, account: AccountId) -> Result<()> {
            let caller = self.env().caller();
            if caller != self.owner {
                self.env().emit_event(TransactionFailed {
                    error: format!("{:?}", Error::PermissionDenied)
                });
                return Err(Error::PermissionDenied);
            }

            self.accounts_blacklisted.insert(account, true);

            self.env().emit_event(AddedBlackList {
                account
            });

            Ok(())
        }

        /// Remove an account from blacklist
        /// 
        /// On success an `RemovedBlackList` event is emitted.
        /// 
        /// # Errors
        /// 
        /// Returns `PermissionDenied` error if caller is not the owner.
        #[ink(message)]
        pub fn remove_account_from_blacklist(&mut self, account: AccountId) -> Result<()> {
            let caller = self.env().caller();
            if caller != self.owner {
                self.env().emit_event(TransactionFailed {
                    error: format!("{:?}", Error::PermissionDenied)
                });
                return Err(Error::PermissionDenied);
            }

            self.accounts_blacklisted.insert(account, false);

            self.env().emit_event(RemovedBlackList {
                account
            });

            Ok(())
        }

        /// Destroy funds of a blacklisted account
        /// 
        /// On success an `DestroyedBlackFunds` event is emitted.
        /// 
        /// # Errors
        /// 
        /// Returns `PermissionDenied` error if caller is not the owner, `AccountNotBlackListed` if the account is not blacklisted
        #[ink(message)]
        pub fn destroy_black_funds(&mut self, account: AccountId) -> Result<()> {
            let caller = self.env().caller();
            if caller != self.owner {
                self.env().emit_event(TransactionFailed {
                    error: format!("{:?}", Error::PermissionDenied)
                });
                return Err(Error::PermissionDenied);
            }

            let blacklisted = self.is_account_blacklisted(account);
            if !blacklisted {
                self.env().emit_event(TransactionFailed {
                    error: format!("{:?}", Error::AccountNotBlackListed)
                });
                return Err(Error::AccountNotBlackListed);
            }

            let dirty_funds = self.balance_of(account);
            self.balances.insert(account, 0);

            let total_supply = &mut self.total_supply;
            let current_supply = Lazy::<Balance>::get(total_supply);
            let new_supply = current_supply - dirty_funds;
            Lazy::<Balance>::set(total_supply, new_supply);

            self.env().emit_event(DestroyedBlackFunds {
                account,
                funds: dirty_funds
            });

            Ok(())
        }

    }

    /// Unit tests
    #[cfg(test)]
    mod tests {
        /// Imports all the definitions from the outer scope so we can use them here.
        use super::*;
        use ink_env::{
            hash::{
                Blake2x256,
                CryptoHash,
                HashOutput,
            },
            Clear,
        };

        type Event = <Entropy as ::ink_lang::BaseEvent>::Type;

        use ink_lang as ink;

        fn encoded_into_hash<T>(entity: &T) -> Hash
            where T: scale::Encode
        {
            let mut result = Hash::clear();
            let len_result = result.as_ref().len();
            let encoded = entity.encode();
            let len_encoded = encoded.len();
            if len_encoded <= len_result {
                result.as_mut()[..len_encoded].copy_from_slice(&encoded);
                return result
            }
            let mut hash_output =
                <<Blake2x256 as HashOutput>::Type as Default>::default();
            <Blake2x256 as CryptoHash>::hash(&encoded, &mut hash_output);
            let copy_len = core::cmp::min(hash_output.len(), len_result);
            result.as_mut()[0..copy_len].copy_from_slice(&hash_output[0..copy_len]);
            result
        }

        fn assert_transfer_event(
            event: &ink_env::test::EmittedEvent,
            expected_from: Option<AccountId>,
            expected_to: Option<AccountId>,
            expected_value: Balance,
        ) {
            let decoded_event = <Event as scale::Decode>::decode(&mut &event.data[..])
                .expect("encountered invalid contract event data buffer");
            if let Event::Transfer(Transfer { from, to, value }) = decoded_event {
                assert_eq!(from, expected_from, "encountered invalid Transfer.from");
                assert_eq!(to, expected_to, "encountered invalid Transfer.to");
                assert_eq!(value, expected_value, "encountered invalid Trasfer.value");
            } else {
                panic!("encountered unexpected event kind: expected a Transfer event")
            }

            let expected_topics = vec![
                encoded_into_hash(b"Entropy::Transfer"),
                encoded_into_hash(&expected_from),
                encoded_into_hash(&expected_to),
                encoded_into_hash(&expected_value),
            ];
            for (n, (actual_topic, expected_topic)) in
                event.topics.iter().zip(expected_topics).enumerate()
            {
                let topic = actual_topic
                    .decode::<Hash>()
                    .expect("encountered invalid topic encoding");
                assert_eq!(topic, expected_topic, "encountered invalid topic at {}", n);
            }
        }

        fn assert_issue_event(
            event: &ink_env::test::EmittedEvent,
            expected_value: Balance,
        ) {
            let decoded_event = <Event as scale::Decode>::decode(&mut &event.data[..])
                .expect("encountered invalid contract event data buffer");
            if let Event::Issue(Issue { amount }) = decoded_event {
                assert_eq!(amount, expected_value, "encountered invalid Issue.amount");
            } else {
                panic!("encountered unexpected event kind: expected an Issue event")
            }

            let expected_topics = vec![
                encoded_into_hash(b"Entropy::Issue"),
                encoded_into_hash(&expected_value),
            ];
            for (n, (actual_topic, expected_topic)) in
                event.topics.iter().zip(expected_topics).enumerate()
            {
                let topic = actual_topic
                    .decode::<Hash>()
                    .expect("encountered invalid topic encoding");
                assert_eq!(topic, expected_topic, "encountered invalid topic at {}", n);
            }
        }

        fn assert_redeem_event(
            event: &ink_env::test::EmittedEvent,
            expected_value: Balance,
        ) {
            let decoded_event = <Event as scale::Decode>::decode(&mut &event.data[..])
                .expect("encountered invalid contract event data buffer");
            if let Event::Redeem(Redeem { amount }) = decoded_event {
                assert_eq!(amount, expected_value, "encountered invalid Redeem.amount");
            } else {
                panic!("encountered unexpected event kind: expected a Redeem event")
            }

            let expected_topics = vec![
                encoded_into_hash(b"Entropy::Redeem"),
                encoded_into_hash(&expected_value),
            ];
            for (n, (actual_topic, expected_topic)) in
                event.topics.iter().zip(expected_topics).enumerate()
            {
                let topic = actual_topic
                    .decode::<Hash>()
                    .expect("encountered invalid topic encoding");
                assert_eq!(topic, expected_topic, "encountered invalid topic at {}", n);
            }
        }

        fn assert_privacy_event(
            event: &ink_env::test::EmittedEvent,
            expected_account: AccountId,
            expected_private: bool,
        ) {
            let decoded_event = <Event as scale::Decode>::decode(&mut &event.data[..])
                .expect("encountered invalid contract event data buffer");
            if let Event::Privacy(Privacy { account, private }) = decoded_event {
                assert_eq!(account, expected_account, "encountered invalid Privacy.account");
                assert_eq!(private, expected_private, "encountered invalid Privacy.private");
            } else {
                panic!("encountered unexpected event kind: expected a Privacy event")
            }

            let expected_topics = vec![
                encoded_into_hash(b"Entropy::Privacy"),
                encoded_into_hash(&expected_account),
                encoded_into_hash(&expected_private),
            ];
            for (n, (actual_topic, expected_topic)) in
                event.topics.iter().zip(expected_topics).enumerate()
            {
                let topic = actual_topic
                    .decode::<Hash>()
                    .expect("encountered invalid topic encoding");
                assert_eq!(topic, expected_topic, "encountered invalid topic at {}", n);
            }
        }

        fn assert_added_blacklist_event(
            event: &ink_env::test::EmittedEvent,
            expected_account: AccountId
        ) {
            let decoded_event = <Event as scale::Decode>::decode(&mut &event.data[..])
                .expect("encountered invalid contract event data buffer");
            if let Event::AddedBlackList(AddedBlackList { account }) = decoded_event {
                assert_eq!(account, expected_account, "encountered invalid AddedBlackList.account");
            } else {
                panic!("encountered unexpected event kind: expected a AddedBlackList event")
            }

            let expected_topics = vec![
                encoded_into_hash(b"Entropy::AddedBlackList"),
                encoded_into_hash(&expected_account)
            ];
            for (n, (actual_topic, expected_topic)) in
                event.topics.iter().zip(expected_topics).enumerate()
            {
                let topic = actual_topic
                    .decode::<Hash>()
                    .expect("encountered invalid topic encoding");
                assert_eq!(topic, expected_topic, "encountered invalid topic at {}", n);
            }
        }

        fn assert_removed_blacklist_event(
            event: &ink_env::test::EmittedEvent,
            expected_account: AccountId
        ) {
            let decoded_event = <Event as scale::Decode>::decode(&mut &event.data[..])
                .expect("encountered invalid contract event data buffer");
            if let Event::RemovedBlackList(RemovedBlackList { account }) = decoded_event {
                assert_eq!(account, expected_account, "encountered invalid RemovedBlackList.account");
            } else {
                panic!("encountered unexpected event kind: expected a RemovedBlackList event")
            }

            let expected_topics = vec![
                encoded_into_hash(b"Entropy::RemovedBlackList"),
                encoded_into_hash(&expected_account)
            ];
            for (n, (actual_topic, expected_topic)) in
                event.topics.iter().zip(expected_topics).enumerate()
            {
                let topic = actual_topic
                    .decode::<Hash>()
                    .expect("encountered invalid topic encoding");
                assert_eq!(topic, expected_topic, "encountered invalid topic at {}", n);
            }
        }

        fn assert_destroyed_black_funds_event(
            event: &ink_env::test::EmittedEvent,
            expected_account: AccountId,
            expected_funds: Balance
        ) {
            let decoded_event = <Event as scale::Decode>::decode(&mut &event.data[..])
                .expect("encountered invalid contract event data buffer");
            if let Event::DestroyedBlackFunds(DestroyedBlackFunds { account, funds }) = decoded_event {
                assert_eq!(account, expected_account, "encountered invalid DestroyedBlackFunds.account");
                assert_eq!(funds, expected_funds, "encountered invalid DestroyedBlackFunds.funds");
            } else {
                panic!("encountered unexpected event kind: expected a DestroyedBlackFunds event")
            }

            let expected_topics = vec![
                encoded_into_hash(b"Entropy::DestroyedBlackFunds"),
                encoded_into_hash(&expected_account)
            ];
            for (n, (actual_topic, expected_topic)) in
                event.topics.iter().zip(expected_topics).enumerate()
            {
                let topic = actual_topic
                    .decode::<Hash>()
                    .expect("encountered invalid topic encoding");
                assert_eq!(topic, expected_topic, "encountered invalid topic at {}", n);
            }
        }

        fn assert_transaction_failed_event(
            event: &ink_env::test::EmittedEvent,
            expected_error: String
        ) {
            let decoded_event = <Event as scale::Decode>::decode(&mut &event.data[..])
                .expect("encountered invalid contract event data buffer");
            if let Event::TransactionFailed(TransactionFailed { error }) = decoded_event {
                assert_eq!(error, error, "encountered invalid TransactionFailed.error");
            } else {
                panic!("encountered unexpected event kind: expected a TransactionFailed event")
            }

            let expected_topics = vec![
                encoded_into_hash(b"Entropy::TransactionFailed"),
                encoded_into_hash(&expected_error)
            ];
            for (n, (actual_topic, expected_topic)) in
                event.topics.iter().zip(expected_topics).enumerate()
            {
                let topic = actual_topic
                    .decode::<Hash>()
                    .expect("encountered invalid topic encoding");
                assert_eq!(topic, expected_topic, "encountered invalid topic at {}", n);
            }
        }

        /// The default constructor does its job.
        #[ink::test]
        fn new_works() {
            // Constructor works.
            let _entropy = Entropy::new(100);

            // Transfer event triggered during initial construction.
            let emitted_events = ink_env::test::recorded_events().collect::<Vec<_>>();
            assert_eq!(1, emitted_events.len());

            assert_transfer_event(
                &emitted_events[0],
                None,
                Some(AccountId::from([0x01; 32])),
                100,
            );
        }

        #[ink::test]
        fn default_works() {
            let entropy = Entropy::default();

            let emitted_events = ink_env::test::recorded_events().collect::<Vec<_>>();
            assert_eq!(1, emitted_events.len());

            // default values
            let default_decimals = 6;
            let default_initial_supply :u128 = u128::pow(10, default_decimals) * 1_000_000;
            let default_name = "Entropy Coin";
            let default_symbol = "ENT";

            assert_transfer_event(
                &emitted_events[0],
                None,
                Some(AccountId::from([0x01; 32])),
                default_initial_supply,
            );
            
            assert_eq!(entropy.total_supply(), default_initial_supply);
            assert_eq!(entropy.name(), default_name);
            assert_eq!(entropy.symbol(), default_symbol);
            assert_eq!(entropy.decimals(), default_decimals);
        }

        /// The total supply was applied.
        #[ink::test]
        fn total_supply_works() {
            // Constructor works.
            let entropy = Entropy::new(100);
            // Transfer event triggered during initial construction.
            let emitted_events = ink_env::test::recorded_events().collect::<Vec<_>>();
            assert_transfer_event(
                &emitted_events[0],
                None,
                Some(AccountId::from([0x01; 32])),
                100,
            );
            // Get the token total supply.
            assert_eq!(entropy.total_supply(), 100);
        }

        /// Get the actual balance of an account.
        #[ink::test]
        fn balance_of_works() {
            // Constructor works
            let entropy = Entropy::new(100);
            // Transfer event triggered during initial construction
            let emitted_events = ink_env::test::recorded_events().collect::<Vec<_>>();
            assert_transfer_event(
                &emitted_events[0],
                None,
                Some(AccountId::from([0x01; 32])),
                100,
            );
            let accounts =
                ink_env::test::default_accounts::<ink_env::DefaultEnvironment>()
                    .expect("Cannot get accounts");
            // Alice owns all the tokens on deployment
            assert_eq!(entropy.balance_of(accounts.alice), 100);
            // Bob does not owns tokens
            assert_eq!(entropy.balance_of(accounts.bob), 0);
        }

        #[ink::test]
        fn transfer_ownership_works() {
            // Constructor works.
            let mut entropy = Entropy::new(100);

            // Transfer event triggered during initial construction.
            let accounts =
                ink_env::test::default_accounts::<ink_env::DefaultEnvironment>()
                    .expect("Cannot get accounts");

            assert_eq!(entropy.balance_of(accounts.alice), 100);

            // Assert owner is alice
            assert_eq!(entropy.owner(), accounts.alice);

            // Transfer ownership to bob
            assert_eq!(entropy.transfer_ownership(accounts.bob), Ok(()));

            // Assert new owner is bob
            assert_eq!(entropy.owner(), accounts.bob);
        }

        #[ink::test]
        fn transfer_works() {
            // Constructor works.
            let mut entropy = Entropy::new(100_000_000);
            // Transfer event triggered during initial construction.
            let accounts =
                ink_env::test::default_accounts::<ink_env::DefaultEnvironment>()
                    .expect("Cannot get accounts");

            assert_eq!(entropy.balance_of(accounts.bob), 0);
            // Alice transfers 20_000_000 tokens to Bob.
            assert_eq!(entropy.transfer(accounts.bob, 20_000_000), Ok(()));
            // Bob owns 20_000_000 tokens.
            assert_eq!(entropy.balance_of(accounts.bob), 20_000_000);
            // Alice remains 80_000_000 tokens.
            assert_eq!(entropy.balance_of(accounts.alice), 80_000_000);

            // Set transaction fee
            assert_eq!(entropy.set_params(10, 50_000_000), Ok(()));
            // Bob transfers 10_000_000 tokens to Charlie. Fee is 10_000_000 * 10 / 10000 = 10_000,
            // so 9_990_000 tokens transferred to Charlie, 10_000 tokens transferred to Alice, who is the contract owner
            assert_eq!(entropy.transfer_from_to(accounts.bob, accounts.charlie, 10_000_000), Ok(()));
            assert_eq!(entropy.balance_of(accounts.bob), 10_000_000);
            assert_eq!(entropy.balance_of(accounts.charlie), 10_000_000 - 10_000);
            assert_eq!(entropy.balance_of(accounts.alice), 80_000_000 + 10_000);
            

            let emitted_events = ink_env::test::recorded_events().collect::<Vec<_>>();
            assert_eq!(emitted_events.len(), 5);
            // Check first transfer event related to Entropy instantiation.
            assert_transfer_event(&emitted_events[0], None, Some(accounts.alice), 100_000_000);
            // Check the second transfer event relating to the actual trasfer.
            assert_transfer_event(&emitted_events[1], Some(accounts.alice), Some(accounts.bob), 20_000_000);
            // Check the 4th fee transfer event (3rd event is the Params event)
            assert_transfer_event(&emitted_events[3], Some(accounts.bob), Some(accounts.alice), 10_000);
            // Check the 5th transfer event to Charlie
            assert_transfer_event(&emitted_events[4], Some(accounts.bob), Some(accounts.charlie), 10_000_000 - 10_000);
        }

        #[ink::test]
        fn invalid_transfer_should_fail() {
            // Constructor works.
            let mut entropy = Entropy::new(100);
            let accounts =
                ink_env::test::default_accounts::<ink_env::DefaultEnvironment>()
                    .expect("Cannot get accounts");

            assert_eq!(entropy.balance_of(accounts.bob), 0);
            // Get contract address.
            let callee = ink_env::account_id::<ink_env::DefaultEnvironment>()
                .unwrap_or([0x0; 32].into());
            // Create call
            let mut data =
                ink_env::test::CallData::new(ink_env::call::Selector::new([0x00; 4])); // balance_of
            data.push_arg(&accounts.bob);
            // Push the new execution context to set Bob as caller
            ink_env::test::push_execution_context::<ink_env::DefaultEnvironment>(
                accounts.bob,
                callee,
                1000000,
                1000000,
                data,
            );

            // Bob fails to transfers 10 tokens to Eve.
            assert_eq!(
                entropy.transfer(accounts.eve, 10),
                Err(Error::InsufficientBalance)
            );
            // Alice owns all the tokens.
            assert_eq!(entropy.balance_of(accounts.alice), 100);
            assert_eq!(entropy.balance_of(accounts.bob), 0);
            assert_eq!(entropy.balance_of(accounts.eve), 0);

            // Transfer event triggered during initial construction.
            let emitted_events = ink_env::test::recorded_events().collect::<Vec<_>>();
            assert_eq!(emitted_events.len(), 2);
            assert_transfer_event(&emitted_events[0], None, Some(AccountId::from([0x01; 32])), 100);
        }

        #[ink::test]
        fn transfer_from_works() {
            // Constructor works.
            let mut entropy = Entropy::new(100);
            // Transfer event triggered during initial construction.
            let accounts =
                ink_env::test::default_accounts::<ink_env::DefaultEnvironment>()
                    .expect("Cannot get accounts");

            // Bob fails to transfer tokens owned by Alice.
            assert_eq!(
                entropy.transfer_from(accounts.alice, accounts.eve, 10),
                Err(Error::InsufficientAllowance)
            );
            // Alice approves Bob for token transfers on her behalf.
            assert_eq!(entropy.approve(accounts.bob, 10), Ok(()));

            // The approve event takes place.
            assert_eq!(ink_env::test::recorded_events().count(), 3);

            // Get contract address.
            let callee = ink_env::account_id::<ink_env::DefaultEnvironment>()
                .unwrap_or([0x0; 32].into());
            // Create call.
            let mut data =
                ink_env::test::CallData::new(ink_env::call::Selector::new([0x00; 4])); // balance_of
            data.push_arg(&accounts.bob);
            // Push the new execution context to set Bob as caller.
            ink_env::test::push_execution_context::<ink_env::DefaultEnvironment>(
                accounts.bob,
                callee,
                1000000,
                1000000,
                data,
            );

            // Bob transfers tokens from Alice to Eve.
            assert_eq!(
                entropy.transfer_from(accounts.alice, accounts.eve, 10),
                Ok(())
            );
            // Eve owns tokens.
            assert_eq!(entropy.balance_of(accounts.eve), 10);

            // Check all transfer events that happened during the previous calls:
            let emitted_events = ink_env::test::recorded_events().collect::<Vec<_>>();
            assert_eq!(emitted_events.len(), 4);
            assert_transfer_event(&emitted_events[0], None, Some(AccountId::from([0x01; 32])), 100);
            // The last event `emitted_events[3]` is an Approve event that we skip checking.
            assert_transfer_event(&emitted_events[3], Some(AccountId::from([0x01; 32])), Some(AccountId::from([0x05; 32])), 10);
        }

        #[ink::test]
        fn allowance_must_not_change_on_failed_transfer() {
            let mut entropy = Entropy::new(100);
            let accounts =
                ink_env::test::default_accounts::<ink_env::DefaultEnvironment>()
                    .expect("Cannot get accounts");

            // Alice approves Bob for token transfers on her behalf.
            let alice_balance = entropy.balance_of(accounts.alice);
            let initial_allowance = alice_balance + 2;
            assert_eq!(entropy.approve(accounts.bob, initial_allowance), Ok(()));

            // Get contract address.
            let callee = ink_env::account_id::<ink_env::DefaultEnvironment>()
                .unwrap_or([0x0; 32].into());
            // Create call.
            let mut data =
                ink_env::test::CallData::new(ink_env::call::Selector::new([0x00; 4])); // balance_of
            data.push_arg(&accounts.bob);
            // Push the new execution context to set Bob as caller.
            ink_env::test::push_execution_context::<ink_env::DefaultEnvironment>(
                accounts.bob,
                callee,
                1000000,
                1000000,
                data,
            );

            // Bob tries to transfer tokens from Alice to Eve.
            let emitted_events_before =
                ink_env::test::recorded_events().collect::<Vec<_>>();
            assert_eq!(
                entropy.transfer_from(accounts.alice, accounts.eve, alice_balance + 1),
                Err(Error::InsufficientBalance)
            );
            // Allowance must have stayed the same
            assert_eq!(
                entropy.allowance(accounts.alice, accounts.bob),
                initial_allowance
            );
            // One more failed event has been emitted
            let emitted_events_after =
                ink_env::test::recorded_events().collect::<Vec<_>>();
            assert_eq!(emitted_events_before.len() + 1, emitted_events_after.len());
        }

        #[ink::test]
        fn issue_works() {
            // Constructor works.
            let mut entropy = Entropy::new(100);

            // Transfer event triggered during initial construction.
            let accounts =
                ink_env::test::default_accounts::<ink_env::DefaultEnvironment>()
                    .expect("Cannot get accounts");

            assert_eq!(entropy.balance_of(accounts.alice), 100);

            // Issue 100 more tokens
            assert_eq!(entropy.issue(100), Ok(()));

            // Check total supply
            assert_eq!(entropy.total_supply(), 200);

            // Check Alice's new balance
            assert_eq!(entropy.balance_of(accounts.alice), 200);

            let emitted_events = ink_env::test::recorded_events().collect::<Vec<_>>();
            assert_eq!(emitted_events.len(), 2);

            // Check first transfer event related to Entropy instantiation.
            assert_transfer_event(&emitted_events[0], None, Some(AccountId::from([0x01; 32])), 100);
            // Check second Issue event
            assert_issue_event(&emitted_events[1], 100);
        }

        #[ink::test]
        fn redeem_works() {
            // Constructor works.
            let mut entropy = Entropy::new(100);

            // Transfer event triggered during initial construction.
            let accounts =
                ink_env::test::default_accounts::<ink_env::DefaultEnvironment>()
                    .expect("Cannot get accounts");

            assert_eq!(entropy.balance_of(accounts.alice), 100);

            // Redeem 50 tokens
            assert_eq!(entropy.redeem(50), Ok(()));

            // Check total supply
            assert_eq!(entropy.total_supply(), 50);

            // Check Alice's new balance
            assert_eq!(entropy.balance_of(accounts.alice), 50);

            let emitted_events = ink_env::test::recorded_events().collect::<Vec<_>>();
            assert_eq!(emitted_events.len(), 2);

            // Check first transfer event related to Entropy instantiation.
            assert_transfer_event(&emitted_events[0], None, Some(AccountId::from([0x01; 32])), 100);
            // Check second Redeem event
            assert_redeem_event(&emitted_events[1], 50);
        }

        #[ink::test]
        fn account_private_works() {
            // Constructor works.
            let mut entropy = Entropy::new(100);

            // Transfer event triggered during initial construction.
            let accounts =
                ink_env::test::default_accounts::<ink_env::DefaultEnvironment>()
                    .expect("Cannot get accounts");

            assert_eq!(entropy.is_account_private(accounts.alice), false);

            // Set Alice as private
            assert_eq!(entropy.set_account_private(accounts.alice, true), Ok(()));

            // Check Alice's privateness
            assert_eq!(entropy.is_account_private(accounts.alice), true);

            // Set Alice's privateness back
            assert_eq!(entropy.set_account_private(accounts.alice, false), Ok(()));

            // Check Alice's privateness again
            assert_eq!(entropy.is_account_private(accounts.alice), false);

            // Check events
            let emitted_events = ink_env::test::recorded_events().collect::<Vec<_>>();
            assert_eq!(emitted_events.len(), 3);

            // Check first transfer event related to Entropy instantiation.
            assert_transfer_event(&emitted_events[0], None, Some(accounts.alice), 100);
            // Check 2nd and 3rd Privacy event
            assert_privacy_event(&emitted_events[1], accounts.alice, true);
            assert_privacy_event(&emitted_events[2], accounts.alice, false);
        }

        #[ink::test]
        fn blacklist_works() {
            // Constructor works.
            let mut entropy = Entropy::new(100);

            // Transfer event triggered during initial construction.
            let accounts =
                ink_env::test::default_accounts::<ink_env::DefaultEnvironment>()
                    .expect("Cannot get accounts");

            assert_eq!(entropy.is_account_blacklisted(accounts.alice), false);
            assert_eq!(entropy.is_account_blacklisted(accounts.bob), false);

            // Alice transfers 10 tokens to bob
            assert_eq!(entropy.transfer(accounts.bob, 10), Ok(()));

            // Destroying bob's funds should fail
            assert_eq!(entropy.destroy_black_funds(accounts.bob), Err(Error::AccountNotBlackListed));

            // Add bob to blacklist
            assert_eq!(entropy.add_account_to_blacklist(accounts.bob), Ok(()));

            // Asset bob is on blacklist
            assert_eq!(entropy.is_account_blacklisted(accounts.bob), true);

            // Destroying bob's funds should now succeed
            assert_eq!(entropy.destroy_black_funds(accounts.bob), Ok(()));

            // Assert totol supply
            assert_eq!(entropy.total_supply(), 90);

            // Remove bob from blacklist
            assert_eq!(entropy.remove_account_from_blacklist(accounts.bob), Ok(()));
            assert_eq!(entropy.is_account_blacklisted(accounts.bob), false);

            // Check events
            let emitted_events = ink_env::test::recorded_events().collect::<Vec<_>>();
            assert_eq!(emitted_events.len(), 6);
            assert_transfer_event(&emitted_events[0], None, Some(accounts.alice), 100);
            assert_transfer_event(&emitted_events[1], Some(accounts.alice), Some(accounts.bob), 10);
            assert_transaction_failed_event(&emitted_events[2], format!("{:?}", Error::AccountNotBlackListed));
            assert_added_blacklist_event(&emitted_events[3], accounts.bob);
            assert_destroyed_black_funds_event(&emitted_events[4], accounts.bob, 10);
            assert_removed_blacklist_event(&emitted_events[5], accounts.bob);
        }

        #[ink::test]
        fn permission_check_works() {
            let mut entropy = Entropy::new(100);
            let accounts = ink_env::test::default_accounts::<ink_env::DefaultEnvironment>().expect("Cannot get accounts");

            // Assert owner is alice
            assert_eq!(entropy.owner(), accounts.alice);

            // Get contract address.
            let callee = ink_env::account_id::<ink_env::DefaultEnvironment>().unwrap_or([0x0; 32].into());

            // Create call.
            let mut data = ink_env::test::CallData::new(ink_env::call::Selector::new([0x00; 4]));
            data.push_arg(&accounts.bob);

            // Push the new execution context to set Bob as caller.
            ink_env::test::push_execution_context::<ink_env::DefaultEnvironment>(accounts.bob, callee, 1000000, 1000000, data);

            // Bob should not have the permission to call privileged apis
            assert_eq!(entropy.transfer_ownership(accounts.charlie), Err(Error::PermissionDenied));
            assert_eq!(entropy.issue(100), Err(Error::PermissionDenied));
            assert_eq!(entropy.redeem(100), Err(Error::PermissionDenied));
            assert_eq!(entropy.set_params(10, 50), Err(Error::PermissionDenied));
            assert_eq!(entropy.set_account_private(accounts.charlie, true), Err(Error::PermissionDenied));
            assert_eq!(entropy.add_account_to_blacklist(accounts.charlie), Err(Error::PermissionDenied));
            assert_eq!(entropy.remove_account_from_blacklist(accounts.charlie), Err(Error::PermissionDenied));
            assert_eq!(entropy.destroy_black_funds(accounts.charlie), Err(Error::PermissionDenied));

            // Transfer ownership to bob
            let mut data = ink_env::test::CallData::new(ink_env::call::Selector::new([0x00; 4]));
            data.push_arg(&accounts.bob);
            ink_env::test::push_execution_context::<ink_env::DefaultEnvironment>(accounts.alice, callee, 1000000, 1000000, data);
            assert_eq!(entropy.transfer_ownership(accounts.bob), Ok(()));
            assert_eq!(entropy.owner(), accounts.bob);

            // Now bob is new owner, should have permission to call privileged apis
            let mut data = ink_env::test::CallData::new(ink_env::call::Selector::new([0x00; 4]));
            data.push_arg(&accounts.bob);
            ink_env::test::push_execution_context::<ink_env::DefaultEnvironment>(accounts.bob, callee, 1000000, 1000000, data);
            assert_eq!(entropy.issue(100), Ok(()));
            assert_eq!(entropy.redeem(100), Ok(()));
            assert_eq!(entropy.set_params(10, 50), Ok(()));
            assert_eq!(entropy.set_account_private(accounts.charlie, true), Ok(()));
            assert_eq!(entropy.add_account_to_blacklist(accounts.charlie), Ok(()));
            assert_eq!(entropy.destroy_black_funds(accounts.charlie), Ok(()));
            assert_eq!(entropy.remove_account_from_blacklist(accounts.charlie), Ok(()));
        }

    }

}
