use actix_web::web::Data;
use actix_web::{middleware, App, HttpServer};
use std::env;
use std::time::Duration;
use tokio::{io, time};

pub(crate) mod components {
    pub mod auth;
    pub mod endpoints;
    pub mod interra;
    pub mod serde_models;
}
use components::endpoints;
use components::interra::InterraTcpClient;

pub async fn run() -> io::Result<()> {
    env::set_var("RUST_LOG", "actix_web=debug,actix_server=info");
    env_logger::init();

    let interra = InterraTcpClient::connect().await?;
    let data = Data::new(interra);
    
    let data_loop = data.clone();
    tokio::spawn(async move {
        loop {
            time::sleep(Duration::from_secs(180)).await;
            if let Err(e) = data_loop.keep_alive().await {
                println!("error with the ol' loop :// {e}");
            }
        }
    });

    let ip = if cfg!(debug_assertions) {
        "localhost:8080"
    } else {
        "0.0.0.0:80"
    };

    HttpServer::new(move || {
        App::new()
            .app_data(data.clone())
            .wrap(middleware::Logger::default())
            .service(endpoints::root)
            .service(endpoints::set_light)
            .service(endpoints::get_lights)
            .service(endpoints::get_light)
            .service(endpoints::restart)
            .service(endpoints::get_ac)
            .service(endpoints::set_ac)
    })
    .bind(ip)?
    .run()
    .await?;

    Ok(())
}
