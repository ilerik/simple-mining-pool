//! Simple mining pool node

extern crate exonum;
extern crate exonum_configuration;
extern crate exonum_time;
extern crate simple_mining_pool;

use exonum::helpers;
use exonum::helpers::fabric::NodeBuilder;
use simple_mining_pool as pool;

fn main() {
    exonum::crypto::init();
    helpers::init_logger().unwrap();

    let node = NodeBuilder::new()
        .with_service(Box::new(exonum_configuration::ServiceFactory))
        .with_service(Box::new(exonum_time::TimeServiceFactory))
        .with_service(Box::new(pool::ServiceFactory));
    node.run();
}
