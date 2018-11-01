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

//! Cryptocurrency implementation example using [exonum](http://exonum.com/).

#![deny(missing_debug_implementations, unsafe_code)]

#[macro_use]
extern crate exonum;
#[macro_use]
extern crate failure;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate frank_jwt as jwt;
#[macro_use]
extern crate serde_json as json;
use exonum::encoding::serialize::json::reexport as serde_json;

pub use schema::CoreSchema;
pub use error::Error;

pub mod error;
pub mod schema;
pub mod api;
pub mod transactions;
pub mod account;

use exonum::api::ServiceApiBuilder;
use exonum::blockchain::{Service, Transaction, TransactionSet};
use exonum::crypto::Hash;
use exonum::encoding::Error as EncodingError;
use exonum::helpers::fabric::{self, Context};
use exonum::messages::RawTransaction;
use exonum::storage::Snapshot;

use transactions::CoreTransactions;

/// Unique service ID.
const SERVICE_ID: u16 = 128;
/// Name of the service.
pub const SERVICE_NAME: &str = "simple";
/// Initial balance of the wallet.
const INITIAL_BALANCE: u64 = 100;

/// Exonum `Service` implementation.
#[derive(Default, Debug)]
pub struct CoreService;

impl Service for CoreService {
    fn service_name(&self) -> &str {
        SERVICE_NAME
    }

    fn service_id(&self) -> u16 {
        SERVICE_ID
    }

    fn state_hash(&self, view: &dyn Snapshot) -> Vec<Hash> {
        let schema = CoreSchema::new(view);
        schema.state_hash()
    }

    fn tx_from_raw(&self, raw: RawTransaction) -> Result<Box<dyn Transaction>, EncodingError> {
        CoreTransactions::tx_from_raw(raw).map(Into::into)
    }

    fn wire_api(&self, builder: &mut ServiceApiBuilder) {
        api::CoreApi::wire(builder);
    }
}

#[derive(Debug)]
pub struct ServiceFactory;

impl fabric::ServiceFactory for ServiceFactory {
    fn service_name(&self) -> &str {
        SERVICE_NAME
    }

    fn make_service(&mut self, _: &Context) -> Box<dyn Service> {
        Box::new(CoreService)
    }
}
