extern crate exonum;
extern crate exonum_configuration;
extern crate exonum_rng;

use exonum::helpers::fabric::NodeBuilder;

use exonum_configuration::ServiceFactory;
use exonum_rng::ExonumRngService;

fn main() {
    exonum::helpers::init_logger().unwrap();
    NodeBuilder::new()
        .with_service(Box::new(ServiceFactory))
        .with_service(Box::new(ExonumRngService::new()))
        .run();
}