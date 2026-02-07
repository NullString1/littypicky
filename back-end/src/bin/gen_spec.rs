use back_end::ApiDoc;
use utoipa::OpenApi;

fn main() {
    println!(
        "{}",
        ApiDoc::openapi()
            .to_pretty_json()
            .expect("Failed to serialize OpenAPI spec")
    );
}
