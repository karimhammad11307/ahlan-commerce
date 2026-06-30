use std::fs;
use std::path::PathBuf;
use utoipa::OpenApi;

fn main() {
    let openapi_json = api::openapi::ApiDoc::openapi().to_pretty_json().unwrap();

    let schema = async_graphql::Schema::build(
        api::graphql::query::QueryRoot,
        api::graphql::mutation::MutationRoot,
        async_graphql::EmptySubscription,
    )
    .finish();

    let graphql_sdl = schema.sdl();

    // Determine the root of the project by going up two directories from apps/api
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| ".".to_string());

    let mut output_dir = PathBuf::from(manifest_dir);
    output_dir.push("..");
    output_dir.push("..");
    output_dir.push("docs");
    output_dir.push("generated");

    fs::create_dir_all(&output_dir).expect("Failed to create docs/generated directory");

    fs::write(output_dir.join("openapi.json"), openapi_json).expect("Failed to write openapi.json");

    fs::write(output_dir.join("schema.graphql"), graphql_sdl)
        .expect("Failed to write schema.graphql");

    println!(
        "Successfully generated API documentation to {:?}",
        output_dir
    );
}
