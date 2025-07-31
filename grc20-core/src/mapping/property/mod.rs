pub mod insert_many;
pub mod insert_one;
pub mod models;

pub use insert_many::insert_many;
pub use insert_one::insert_one;
pub use models::{DataType, Property};
