pub mod mutation;
pub mod query;
pub mod types;

use crate::AppState;
use async_graphql::{EmptySubscription, Schema};

pub type AppSchema = Schema<query::QueryRoot, mutation::MutationRoot, EmptySubscription>;

pub fn build_schema(state: AppState) -> AppSchema {
    Schema::build(query::QueryRoot, mutation::MutationRoot, EmptySubscription)
        .data(state)
        .finish()
}
