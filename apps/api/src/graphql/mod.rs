pub mod types;
pub mod query;
pub mod mutation;

use async_graphql::{EmptySubscription, Schema};
use crate::AppState;

pub type AppSchema = Schema<query::QueryRoot, mutation::MutationRoot, EmptySubscription>;

pub fn build_schema(state: AppState) -> AppSchema {
    Schema::build(
        query::QueryRoot,
        mutation::MutationRoot,
        EmptySubscription,
    )
    .data(state)
    .finish()
}
