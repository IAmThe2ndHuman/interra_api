#[actix_web::main]
async fn main() {
    if let Err(e) = interra_api::run().await {
        eprintln!("App error: {e}")
    }
}
