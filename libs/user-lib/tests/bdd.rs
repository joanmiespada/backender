mod support;

use cucumber::World;
use support::world::TestWorld;

// Import step definitions to register them with cucumber
#[allow(unused_imports)]
use support::steps::given;
#[allow(unused_imports)]
use support::steps::when_user;
#[allow(unused_imports)]
use support::steps::when_role;
#[allow(unused_imports)]
use support::steps::then_user;
#[allow(unused_imports)]
use support::steps::then_role;

#[tokio::main]
async fn main() {
    TestWorld::run("tests/features").await;
}
