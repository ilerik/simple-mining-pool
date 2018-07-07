//! Account.

use exonum::crypto::{Hash, PublicKey};

encoding_struct! {
    /// Account information stored in the database.
    struct Account {
        pub_key:            &PublicKey,
        name:               &str,
        balance:            u64,
        history_len:        u64,
        history_hash:       &Hash,
        //access_log_hash:    &Hash,
    }
}

impl Account {
    /// Returns a copy of this account with updated balance.
    pub fn set_balance(self, balance: u64, history_hash: &Hash) -> Self {
        Self::new(
            self.pub_key(),
            self.name(),
            balance,
            self.history_len() + 1,
            history_hash,
        )
    }
}
