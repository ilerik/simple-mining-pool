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

//use serde_json;

use exonum::api::{self, ServiceApiBuilder, ServiceApiState};
use exonum::api::Result as ExonumApiResult;
use exonum::blockchain::{self, BlockProof, Transaction, TransactionSet};
use exonum::crypto::{Hash, PublicKey};
use exonum::helpers::Height;
use exonum::node::TransactionSend;
use exonum::storage::{ListProof, MapProof};

use transactions::CoreTransactions;
use account::Account;
use {CoreSchema, CORE_SERVICE_ID};

/// The structure describes the query parameters for the `get_wallet` endpoint.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct AccountQuery {
    /// Public key of the queried wallet.
    pub pub_key: PublicKey,
}

/// The structure returned by the REST API.
#[derive(Debug, Serialize, Deserialize)]
pub struct TransactionResponse {
    /// Hash of the transaction.
    pub tx_hash: Hash,
}

/// Proof of existence for specific account.
#[derive(Debug, Serialize, Deserialize)]
pub struct AccountProof {
    /// Proof to the whole database table.
    pub to_table: MapProof<Hash, Hash>,
    /// Proof to the specific account in this table.
    pub to_account: MapProof<PublicKey, Account>,
}

/// Account history.
#[derive(Debug, Serialize, Deserialize)]
pub struct AccountHistory {
    pub proof: ListProof<Hash>,
    pub transactions: Vec<CoreTransactions>,
}

/// Account information.
#[derive(Debug, Serialize, Deserialize)]
pub struct AccountInfo {
    pub block_proof: BlockProof,
    pub account_proof: AccountProof,
    pub account_history: Option<AccountHistory>,
}

// TODO: Add documentation. (ECR-1638)
/// Public service API description.
#[derive(Debug, Clone, Copy)]
pub struct CoreApi;

impl CoreApi
{
    /// Helper function reading blockchain state
    ///fetch account info
    /// meaning proofs, history
    fn get_account_info(state: &ServiceApiState, query: AccountQuery) -> ExonumApiResult<AccountInfo> {
        let snapshot = state.snapshot();
        let general_schema = blockchain::Schema::new(&snapshot);
        let core_schema = CoreSchema::new(&snapshot);

        let max_height = general_schema.block_hashes_by_height().len() - 1;

        let block_proof = general_schema
            .block_and_precommits(Height(max_height))
            .unwrap();

        let to_table: MapProof<Hash, Hash> =
            general_schema.get_proof_to_service_table(CORE_SERVICE_ID, 0);

        let to_account: MapProof<PublicKey, Account> = core_schema.accounts().get_proof(query.pub_key);

        let account_proof = AccountProof {
            to_table,
            to_account,
        };

        let account = core_schema.account(&query.pub_key);

        let account_history = account.map(|_| {
            let history = core_schema.account_history(&query.pub_key);
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
    pub fn post_transaction(
        state: &ServiceApiState,
        query: CoreTransactions,
    ) -> api::Result<TransactionResponse> {
        let transaction: Box<dyn Transaction> = query.into();
        let tx_hash = transaction.hash();
        state.sender().send(transaction)?;
        Ok(TransactionResponse { tx_hash })
    }

    /// Endpoint for getting a single account.
    pub fn get_account(state: &ServiceApiState, query: AccountQuery) -> api::Result<Account> {
        let snapshot = state.snapshot();
        let schema = CoreSchema::new(snapshot);
        schema
            .account(&query.pub_key)
            .ok_or_else(|| api::Error::NotFound("\"Account not found\"".to_owned()))
    }

    pub fn wire(builder: &mut ServiceApiBuilder) {
        builder
            .public_scope()
            .endpoint("v1/accounts/info", Self::get_account_info)
            .endpoint("v1/accounts", Self::get_account)
            .endpoint_mut("v1/transaction", Self::post_transaction);
    }
}