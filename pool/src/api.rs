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

//! Cryptocurrency API.

use bodyparser;
use iron::prelude::*;
use router::Router;
use serde_json;

use exonum::api::{Api, ApiError};
use exonum::blockchain::{self, BlockProof, Blockchain, Transaction, TransactionSet};
use exonum::crypto::{Hash, PublicKey};
use exonum::helpers::Height;
use exonum::node::TransactionSend;
use exonum::storage::{ListProof, MapProof};

use std::fmt;

use transactions::CoreTransactions;
use account::Account;
use {CoreSchema, CORE_SERVICE_ID};

/// The structure returned by the REST API.
#[derive(Debug, Serialize, Deserialize)]
pub struct TransactionResponse {
    /// Hash of the transaction.
    pub tx_hash: Hash,
}

/// Proof of existence for specific account.
#[derive(Debug, Serialize)]
pub struct AccountProof {
    /// Proof to the whole database table.
    to_table: MapProof<Hash, Hash>,
    /// Proof to the specific account in this table.
    to_account: MapProof<PublicKey, Account>,
}

/// Account history.
#[derive(Debug, Serialize)]
pub struct AccountHistory {
    proof: ListProof<Hash>,
    transactions: Vec<CoreTransactions>,
}

/// Account information.
#[derive(Debug, Serialize)]
pub struct AccountInfo {
    block_proof: BlockProof,
    account_proof: AccountProof,
    account_history: Option<AccountHistory>,
}

/// TODO: Add documentation.
#[derive(Clone)]
pub struct CoreApi<T: TransactionSend + Clone> {
    /// Exonum blockchain.
    pub blockchain: Blockchain,
    /// Channel for transactions.
    pub channel: T,
}

impl<T> CoreApi<T>
where
    T: TransactionSend + Clone + 'static,
{

    // Helper function reading blockchain state
    fn account_info(&self, pub_key: &PublicKey) -> Result<AccountInfo, ApiError> {
        let view = self.blockchain.snapshot();
        let general_schema = blockchain::Schema::new(&view);
        let mut view = self.blockchain.fork();
        let core_schema = CoreSchema::new(&mut view);

        let max_height = general_schema.block_hashes_by_height().len() - 1;

        let block_proof = general_schema
            .block_and_precommits(Height(max_height))
            .unwrap();

        let to_table: MapProof<Hash, Hash> =
            general_schema.get_proof_to_service_table(CORE_SERVICE_ID, 0);

        let to_account: MapProof<PublicKey, Account> = core_schema.accounts().get_proof(*pub_key);

        let account_proof = AccountProof {
            to_table,
            to_account,
        };

        let account = core_schema.account(pub_key);

        let account_history = account.map(|_| {
            let history = core_schema.account_history(pub_key);
            let proof = history.get_range_proof(0, history.len());

            let transactions: Vec<CoreTransactions> = history
                .iter()
                .map(|record| general_schema.transactions().get(&record).unwrap())
                .map(|raw| CoreTransactions::tx_from_raw(raw).unwrap())
                .collect::<Vec<_>>();

            AccountHistory {
                proof,
                transactions,
            }
        });

        Ok(AccountInfo {
            block_proof,
            account_proof,
            account_history,
        })
    }

    // Relay any transaction
    fn wire_post_transaction(self, router: &mut Router) {
        let transaction = move |req: &mut Request| -> IronResult<Response> {
            match req.get::<bodyparser::Struct<CoreTransactions>>() {
                Ok(Some(transaction)) => {
                    let transaction: Box<Transaction> = transaction.into();
                    let tx_hash = transaction.hash();
                    self.channel.send(transaction).map_err(ApiError::from)?;
                    let json = TransactionResponse { tx_hash };
                    self.ok_response(&serde_json::to_value(&json).unwrap())
                }
                Ok(None) => Err(ApiError::BadRequest("Empty request body".into()))?,
                Err(e) => Err(ApiError::BadRequest(e.to_string()))?,
            }
        };
        router.post("/v1/transaction", transaction, "post_transaction");
    }

    // Relay signin transaction 

    // Fetch account info
    fn wire_account_info(self, router: &mut Router) {
        let account_info = move |req: &mut Request| -> IronResult<Response> {
            let pub_key: PublicKey = self.url_fragment(req, "pubkey")?;
            let info = self.account_info(&pub_key)?;
            self.ok_response(&serde_json::to_value(&info).unwrap())
        };
        router.get("/v1/accounts/info/:pubkey", account_info, "account_info");
    }

    // Fetch account data
    fn wire_account(self, router: &mut Router) {
        let account = move |req: &mut Request| -> IronResult<Response> {
            let pub_key: PublicKey = self.url_fragment(req, "pubkey")?;
            let view = self.blockchain.snapshot();
            let schema = CoreSchema::new(view);
            if let Some(account) = schema.account(&pub_key) {
                self.ok_response(&serde_json::to_value(&account).unwrap())
            } else {
                self.not_found_response(&serde_json::to_value("Account not found").unwrap())
            }
        };
        router.get("/v1/accounts/:pubkey", account, "account");
    }
}

impl<T: TransactionSend + Clone> fmt::Debug for CoreApi<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "CoreApi {{}}")
    }
}

impl<T> Api for CoreApi<T>
where
    T: 'static + TransactionSend + Clone,
{
    fn wire(&self, router: &mut Router) {
        self.clone().wire_post_transaction(router);
        self.clone().wire_account_info(router);
        self.clone().wire_account(router);
    }
}
