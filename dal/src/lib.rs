mod entity;

use rand::Rng;
pub use entity::*;

mod dal_mysql;
pub use dal_mysql::*;

mod error;
pub use error::*;

fn generate_id(len: usize) -> String {
    rand::thread_rng().sample_iter(rand::distributions::Alphanumeric).take(len).map(char::from).collect()
}