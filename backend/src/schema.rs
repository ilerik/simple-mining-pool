// Copyright 2018 The Exonum Team
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//   http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use exonum::crypto::{hash, Hash, PublicKey};
use exonum::storage::{Fork, ProofListIndex, ProofMapIndex, Snapshot};

use INITIAL_BALANCE;
use account::Account;

/// Database schema for the cryptocurrency.
#[derive(Debug)]
pub struct CoreSchema<T> {
    view: T,
}

impl<T> AsMut<T> for CoreSchema<T> {
    fn as_mut(&mut self) -> &mut T {
        &mut self.view
    }
}

impl<T> CoreSchema<T>
where
    T: AsRef<dyn Snapshot>,
{
    /// Constructs schema from the database view.
    pub fn new(view: T) -> Self {
        CoreSchema { view }
    }

    /// Returns `MerklePatriciaTable` with accounts.
    pub fn accounts(&self) -> ProofMapIndex<&T, PublicKey, Account> {
        ProofMapIndex::new("simple_mining_pool.accounts", &self.view)
    }

    /// Returns history of the account with the given public key.
    pub fn account_history(&self, public_key: &PublicKey) -> ProofListIndex<&T, Hash> {
        ProofListIndex::new_in_family("simple_mining_pool.account_history", public_key, &self.view)
    }

    /// Returns account data for the given public key.
    pub fn account(&self, pub_key: &PublicKey) -> Option<Account> {
        self.accounts().get(pub_key)
    }

    /// Returns database state hash.
    pub fn state_hash(&self) -> Vec<Hash> {
        vec![self.accounts().merkle_root()]
    }
}

/// Implementation of mutable methods.
impl<'a> CoreSchema<&'a mut Fork> {
    /// Returns mutable `MerklePatriciaTable` with accounts.
    pub fn accounts_mut(&mut self) -> ProofMapIndex<&mut Fork, PublicKey, Account> {
        ProofMapIndex::new("simple_mining_pool.accounts", &mut self.view)
    }

    /// Returns history for the wallet by the given public key.
    pub fn account_history_mut(
        &mut self,
        public_key: &PublicKey,
    ) -> ProofListIndex<&mut Fork, Hash> {
        ProofListIndex::new_in_family("simple_mining_pool.account_history", public_key, &mut self.view)
    }

    /// Increase balance of the account and append new record to its history.
    ///
    /// Panics if there is no account with given public key.
    pub fn increase_account_balance(&mut self, account: Account, amount: u64, transaction: &Hash) {
        let account = {
            let mut history = self.account_history_mut(account.pub_key());
            history.push(*transaction);
            let history_hash = history.merkle_root();
            let balance = account.balance();
            account.set_balance(balance + amount, &history_hash)
        };
        self.accounts_mut().put(account.pub_key(), account.clone());
    }

    /// Decrease balance of the account and append new record to its history.
    ///
    /// Panics if there is no account with given public key.
    pub fn decrease_account_balance(&mut self, account: Account, amount: u64, transaction: &Hash) {
        let account = {
            let mut history = self.account_history_mut(account.pub_key());
            history.push(*transaction);
            let history_hash = history.merkle_root();
            let balance = account.balance();
            account.set_balance(balance - amount, &history_hash)
        };
        self.accounts_mut().put(account.pub_key(), account.clone());
    }

    /// Create new account and append first record to its history.
    pub fn create_account(&mut self, key: &PublicKey, name: &str, transaction: &Hash) {
        let account = {
            let mut history = self.account_history_mut(key);
            history.push(*transaction);
            let history_hash = history.merkle_root();
            Account::new(key, name, INITIAL_BALANCE, &Hash::default(), history.len(), &history_hash)
        };
        self.accounts_mut().put(key, account);
    }

    /// Sign in into the system and change account state to incorporate proof of authenticity
    pub fn sign_into_account(&mut self, account: Account, token: &str, transaction: &Hash) {
        let account = {
            let mut history = self.account_history_mut(account.pub_key());
            history.push(*transaction);
            let history_hash = history.merkle_root();
            let token_hash = hash(token.as_bytes());
            account.set_access_token(&token_hash, &history_hash)
        };
        self.accounts_mut().put(account.pub_key(), account.clone());
    }
}
