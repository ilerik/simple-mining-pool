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

//! These are tests concerning the API of the cryptocurrency service. See `tx_logic.rs`
//! for tests focused on the business logic of transactions.
//!
//! Note how API tests predominantly use `TestKitApi` to send transactions and make assertions
//! about the storage state.

#[macro_use]
extern crate assert_matches;
extern crate exonum;
extern crate simple_mining_pool;
extern crate exonum_testkit;
#[macro_use]
extern crate serde_json;

use exonum::api::node::public::explorer::TransactionQuery;
use exonum::crypto::{self, CryptoHash, Hash, PublicKey, SecretKey};
use exonum_testkit::{ApiKind, TestKit, TestKitApi, TestKitBuilder};

// Import data types used in tests from the crate where the service is defined.
use simple_mining_pool::CoreService;
use simple_mining_pool::transactions::{CreateAccount, Transfer, SignIn};
use simple_mining_pool::api::{AccountInfo, AccountQuery};
use simple_mining_pool::account::Account;

// Imports shared test constants.
use constants::{ALICE_NAME, BOB_NAME};

mod constants;

/// Check that the wallet creation transaction works when invoked via API.
#[test]
fn test_create_account() {
    let (mut testkit, api) = create_testkit();
    // Create and send a transaction via API
    let (tx, _) = api.create_account(ALICE_NAME);
    testkit.create_block();
    api.assert_tx_status(tx.hash(), &json!({ "type": "success" }));

    // Check that the user indeed is persisted by the service.
    let account = api.get_account(*tx.pub_key()).unwrap();
    assert_eq!(account.pub_key(), tx.pub_key());
    assert_eq!(account.name(), tx.name());
    assert_eq!(account.balance(), 100);
}

#[test]
fn test_sing_into_account() {
    let (mut testkit, api) = create_testkit();
    // Create and send a transaction via API
    let (tx, key) = api.create_account(ALICE_NAME);
    testkit.create_block();
    api.assert_tx_status(tx.hash(), &json!({ "type": "success" }));

    // Check that the user indeed is persisted by the service.
    let account = api.get_account(*tx.pub_key()).unwrap();
    assert_eq!(account.pub_key(), tx.pub_key());
    assert_eq!(account.name(), tx.name());
    assert_eq!(account.balance(), 100);
    assert_eq!(account.access_token_hash(), &Hash::default());

    // Create and send a transaction via API
    let pubkey = account.pub_key();
    let tx = api.sign_into_account(&pubkey, ALICE_NAME, &key);
    testkit.create_block();
    api.assert_tx_status(tx.hash(), &json!({ "type": "success" }));

    // Check that the user indeed is persisted by the service.
    let account = api.get_account(*tx.pub_key()).unwrap();
    assert_eq!(account.pub_key(), tx.pub_key());
    assert_eq!(account.name(), tx.name());
    assert_ne!(account.access_token_hash(), &Hash::default());
    
}


/// Check that the transfer transaction works as intended.
#[test]
fn test_transfer() {
    // Create 2 wallets.
    let (mut testkit, api) = create_testkit();
    let (tx_alice, key_alice) = api.create_account(ALICE_NAME);
    let (tx_bob, _) = api.create_account(BOB_NAME);
    testkit.create_block();
    api.assert_tx_status(tx_alice.hash(), &json!({ "type": "success" }));
    api.assert_tx_status(tx_bob.hash(), &json!({ "type": "success" }));

    // Check that the initial Alice's and Bob's balances persisted by the service.
    let account = api.get_account(*tx_alice.pub_key()).unwrap();
    assert_eq!(account.balance(), 100);
    let account = api.get_account(*tx_bob.pub_key()).unwrap();
    assert_eq!(account.balance(), 100);

    // Transfer funds by invoking the corresponding API method.
    let tx = Transfer::new(
        tx_alice.pub_key(),
        tx_bob.pub_key(),
        10, // transferred amount
        0,  // seed
        &key_alice,
    );
    api.transfer(&tx);
    testkit.create_block();
    api.assert_tx_status(tx.hash(), &json!({ "type": "success" }));

    // After the transfer transaction is included into a block, we may check new wallet
    // balances.
    let account = api.get_account(*tx_alice.pub_key()).unwrap();
    assert_eq!(account.balance(), 90);
    let account = api.get_account(*tx_bob.pub_key()).unwrap();
    assert_eq!(account.balance(), 110);
}

/// Check that a transfer from a non-existing wallet fails as expected.
#[test]
fn test_transfer_from_nonexisting_account() {
    let (mut testkit, api) = create_testkit();

    let (tx_alice, key_alice) = api.create_account(ALICE_NAME);
    let (tx_bob, _) = api.create_account(BOB_NAME);
    // Do not commit Alice's transaction, so Alice's wallet does not exist
    // when a transfer occurs.
    testkit.create_block_with_tx_hashes(&[tx_bob.hash()]);

    api.assert_no_account(*tx_alice.pub_key());
    let account = api.get_account(*tx_bob.pub_key()).unwrap();
    assert_eq!(account.balance(), 100);

    let tx = Transfer::new(
        tx_alice.pub_key(),
        tx_bob.pub_key(),
        10, // transfer amount
        0,  // seed
        &key_alice,
    );
    api.transfer(&tx);
    testkit.create_block_with_tx_hashes(&[tx.hash()]);
    api.assert_tx_status(
        tx.hash(),
        &json!({ "type": "error", "code": 1, "description": "Sender doesn't exist" }),
    );

    // Check that Bob's balance doesn't change.
    let account = api.get_account(*tx_bob.pub_key()).unwrap();
    assert_eq!(account.balance(), 100);
}

/// Check that a transfer to a non-existing wallet fails as expected.
#[test]
fn test_transfer_to_nonexisting_account() {
    let (mut testkit, api) = create_testkit();

    let (tx_alice, key_alice) = api.create_account(ALICE_NAME);
    let (tx_bob, _) = api.create_account(BOB_NAME);
    // Do not commit Bob's transaction, so Bob's wallet does not exist
    // when a transfer occurs.
    testkit.create_block_with_tx_hashes(&[tx_alice.hash()]);

    let account = api.get_account(*tx_alice.pub_key()).unwrap();
    assert_eq!(account.balance(), 100);
    api.assert_no_account(*tx_bob.pub_key());

    let tx = Transfer::new(
        tx_alice.pub_key(),
        tx_bob.pub_key(),
        10, // transfer amount
        0,  // seed
        &key_alice,
    );
    api.transfer(&tx);
    testkit.create_block_with_tx_hashes(&[tx.hash()]);
    api.assert_tx_status(
        tx.hash(),
        &json!({ "type": "error", "code": 2, "description": "Receiver doesn't exist" }),
    );

    // Check that Alice's balance doesn't change.
    let account = api.get_account(*tx_alice.pub_key()).unwrap();
    assert_eq!(account.balance(), 100);
}

/// Check that an overcharge does not lead to changes in sender's and receiver's balances.
#[test]
fn test_transfer_overcharge() {
    let (mut testkit, api) = create_testkit();

    let (tx_alice, key_alice) = api.create_account(ALICE_NAME);
    let (tx_bob, _) = api.create_account(BOB_NAME);
    testkit.create_block();

    // Transfer funds. The transfer amount (110) is more than Alice has (100).
    let tx = Transfer::new(
        tx_alice.pub_key(),
        tx_bob.pub_key(),
        110, // transfer amount
        0,   // seed
        &key_alice,
    );
    api.transfer(&tx);
    testkit.create_block();
    api.assert_tx_status(
        tx.hash(),
        &json!({ "type": "error", "code": 3, "description": "Insufficient currency amount" }),
    );

    let account = api.get_account(*tx_alice.pub_key()).unwrap();
    assert_eq!(account.balance(), 100);
    let account = api.get_account(*tx_bob.pub_key()).unwrap();
    assert_eq!(account.balance(), 100);
}

#[test]
#[should_panic(expected = "Unable to serialize query.")]//probably, more reliable test should be written
fn test_malformed_account_request() {
    let (_testkit, api) = create_testkit();

    let
    account_info = api.inner
        .public(ApiKind::Service("simple_mining_pool"))
        .query(&Box::new("c0ffee"))
        .get::<AccountInfo>("v1/accounts/info")
    ;
}

#[test]
fn test_unknown_account_request() {
    let (_testkit, api) = create_testkit();

    // Transaction is sent by API, but isn't committed.
    let (tx, _) = api.create_account(ALICE_NAME);

    api.assert_no_account(*tx.pub_key());
}

/// Wrapper for the cryptocurrency service API allowing to easily use it
/// (compared to `TestKitApi` calls).
struct CoreApi {
    pub inner: TestKitApi,
}

impl CoreApi {
    /// Generates a wallet creation transaction with a random key pair, sends it over HTTP,
    /// and checks the synchronous result (i.e., the hash of the transaction returned
    /// within the response).
    /// Note that the transaction is not immediately added to the blockchain, but rather is put
    /// to the pool of unconfirmed transactions.
    fn create_account(&self, name: &str) -> (CreateAccount, SecretKey) {
        let (pubkey, key) = crypto::gen_keypair();
        // Create a pre-signed transaction
        let tx = CreateAccount::new(&pubkey, name, &key);

        let tx_info: serde_json::Value = self.inner
            .public(ApiKind::Service("simple_mining_pool"))
            .query(&tx)
            .post("v1/transaction")
            .unwrap();
        assert_eq!(tx_info, json!({ "tx_hash": tx.hash() }));
        (tx, key)
    }

    fn get_account(&self, pub_key: PublicKey) -> Option<Account> {
        let account_info = self.inner
            .public(ApiKind::Service("simple_mining_pool"))
            .query(&AccountQuery { pub_key })
            .get::<AccountInfo>("v1/accounts/info")
            .unwrap();

        let to_account = account_info.account_proof.to_account.check().unwrap();
        to_account
            .all_entries()
            .iter()
            .find(|(ref k, _)| **k == pub_key)
            .and_then(|tuple| tuple.1)
            .cloned()
    }

    /// Create signin transaction given user credentials and private key
    fn sign_into_account(&self, pubkey: &PublicKey, name: &str, key: &SecretKey) -> (SignIn) {
        // Create a pre-signed transaction
        let tx = SignIn::new(&pubkey, name, &key);

        let tx_info: serde_json::Value = self.inner
            .public(ApiKind::Service("simple_mining_pool"))
            .query(&tx)
            .post("v1/transaction")
            .unwrap();
        assert_eq!(tx_info, json!({ "tx_hash": tx.hash() }));
        (tx)
    }

    /// Sends a transfer transaction over HTTP and checks the synchronous result.
    fn transfer(&self, tx: &Transfer) {
        let tx_info: serde_json::Value = self.inner
            .public(ApiKind::Service("simple_mining_pool"))
            .query(&tx)
            .post("v1/transaction")
            .unwrap();
        assert_eq!(tx_info, json!({ "tx_hash": tx.hash() }));
    }

    /// Asserts that a wallet with the specified public key is not known to the blockchain.
    fn assert_no_account(&self, pub_key: PublicKey) {
        let account_info: AccountInfo = self.inner
            .public(ApiKind::Service("simple_mining_pool"))
            .query(&AccountQuery { pub_key })
            .get("v1/accounts/info")
            .unwrap();

        let to_account = account_info.account_proof.to_account.check().unwrap();
        assert!(
            to_account
                .missing_keys()
                .iter()
                .find(|v| ***v == pub_key)
                .is_some()
        )
    }

    /// Asserts that the transaction with the given hash has a specified status.
    fn assert_tx_status(&self, tx_hash: Hash, expected_status: &serde_json::Value) {
        let info: serde_json::Value = self.inner
            .public(ApiKind::Explorer)
            .query(&TransactionQuery::new(tx_hash))
            .get("v1/transactions")
            .unwrap();

        if let serde_json::Value::Object(mut info) = info {
            let tx_status = info.remove("status").unwrap();
            assert_eq!(tx_status, *expected_status);
        } else {
            panic!("Invalid transaction info format, object expected");
        }
    }
}

/// Creates a testkit together with the API wrapper defined above.
fn create_testkit() -> (TestKit, CoreApi) {
    let testkit = TestKitBuilder::validator()
        .with_service(CoreService)
        .create();
    let api = CoreApi {
        inner: testkit.api(),
    };
    (testkit, api)
}
