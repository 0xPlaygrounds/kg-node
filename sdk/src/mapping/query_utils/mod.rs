use crate::error::DatabaseError;

pub mod attributes_filter;
pub mod edge_filter;
pub mod order_by;
pub mod prop_filter;
pub mod query_part;
pub mod version_filter;

pub use attributes_filter::AttributeFilter;
pub use edge_filter::EdgeFilter;
use futures::Stream;
pub use order_by::{FieldOrderBy, OrderDirection};
pub use prop_filter::PropFilter;
pub use query_part::QueryPart;
pub use version_filter::VersionFilter;

pub trait Query<T> {
    fn send(self) -> impl std::future::Future<Output = Result<T, DatabaseError>>;
}

pub trait QueryStream<T> {
    fn send(
        self,
    ) -> impl std::future::Future<
        Output = Result<impl Stream<Item = Result<T, DatabaseError>>, DatabaseError>,
    >;
}
