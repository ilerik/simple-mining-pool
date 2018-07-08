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

use exonum::blockchain::{ExecutionError, ExecutionResult, Transaction};
use exonum::crypto::{CryptoHash, PublicKey};
use exonum::messages::Message;
use exonum::storage::Fork;

use CORE_SERVICE_ID;
use schema::CoreSchema;

/// Error codes emitted by wallet transactions during execution.
#[derive(Debug, Fail)]
#[repr(u8)]
pub enum Error {
    /// Account already exists.
    ///
    /// Can be emitted by `CreateAccount`.
    #[fail(display = "Account already exists")]
    AccountAlreadyExists = 0,

    /// Sender doesn't exist.
    ///
    /// Can be emitted by `Transfer`.
    #[fail(display = "Sender doesn't exist")]
    SenderNotFound = 1,

    /// Receiver doesn't exist.
    ///
    /// Can be emitted by `Transfer` or `Issue`.
    #[fail(display = "Receiver doesn't exist")]
    ReceiverNotFound = 2,

    /// Insufficient currency amount.
    ///
    /// Can be emitted by `Transfer`.
    #[fail(display = "Insufficient currency amount")]
    InsufficientCurrencyAmount = 3,

    /// Sign in failed
    ///
    /// Can be emitted by `SignIn`.
    #[fail(display = "Sign in failed")]
    AccountDoesntExist = 4,
    
}

impl From<Error> for ExecutionError {
    fn from(value: Error) -> ExecutionError {
        let description = format!("{}", value);
        ExecutionError::with_description(value as u8, description)
    }
}

transactions! {
    pub CoreTransactions {
        const SERVICE_ID = CORE_SERVICE_ID;

        /// Transfer `amount` of the currency from one wallet to another.
        struct Transfer {
            from:    &PublicKey,
            to:      &PublicKey,
            amount:  u64,
            seed:    u64,
        }

        /// Issue `amount` of the currency to the `wallet`.
        struct Issue {
            pub_key:  &PublicKey,
            amount:  u64,
            seed:    u64,
        }

        /// Create account with the given `name`.
        struct CreateAccount {
            pub_key:            &PublicKey,
            name:               &str,
        }

        /// Create wallet with the given `name`.
        struct SignIn {
            pub_key:            &PublicKey,
            name:               &str,
        }
    }
}

impl Transaction for Transfer {
    fn verify(&self) -> bool {
        self.verify_signature(self.from())
    }

    fn execute(&self, fork: &mut Fork) -> ExecutionResult {
        let mut schema = CoreSchema::new(fork);

        let from = self.from();
        let to = self.to();
        let hash = self.hash();
        let amount = self.amount();

        let sender = schema.account(from).ok_or(Error::SenderNotFound)?;

        let receiver = schema.account(to).ok_or(Error::ReceiverNotFound)?;

        if sender.balance() < amount {
            Err(Error::InsufficientCurrencyAmount)?
        }

        schema.decrease_account_balance(sender, amount, &hash);
        schema.increase_account_balance(receiver, amount, &hash);

        Ok(())
    }
}

impl Transaction for Issue {
    fn verify(&self) -> bool {
        self.verify_signature(self.pub_key())
    }

    fn execute(&self, fork: &mut Fork) -> ExecutionResult {
        let mut schema = CoreSchema::new(fork);
        let pub_key = self.pub_key();
        let hash = self.hash();

        if let Some(account) = schema.account(pub_key) {
            let amount = self.amount();
            schema.increase_account_balance(account, amount, &hash);
            Ok(())
        } else {
            Err(Error::ReceiverNotFound)?
        }
    }
}

impl Transaction for CreateAccount {
    fn verify(&self) -> bool {
        self.verify_signature(self.pub_key())
    }

    fn execute(&self, fork: &mut Fork) -> ExecutionResult {
        let mut schema = CoreSchema::new(fork);
        let pub_key = self.pub_key();
        let hash = self.hash();

        if schema.account(pub_key).is_none() {
            let name = self.name();
            schema.create_account(pub_key, name, &hash);
            Ok(())
        } else {
            Err(Error::AccountAlreadyExists)?
        }
    }
}

impl Transaction for SignIn {
    fn verify(&self) -> bool {
        self.verify_signature(self.pub_key())
    }

    fn execute(&self, fork: &mut Fork) -> ExecutionResult {
        let mut schema = CoreSchema::new(fork);
        let pub_key = self.pub_key();
        let hash = self.hash();

        if let Some(account) = schema.account(pub_key) {
            let name = self.name();
            //schema.create_account(pub_key, name, &hash);
            Ok(())
        } else {
            Err(Error::AccountDoesntExist)?
        }
    }
}
