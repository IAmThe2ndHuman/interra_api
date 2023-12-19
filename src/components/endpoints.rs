use crate::components::auth::Authorized;
use crate::components::interra::InterraTcpClient;
use crate::components::serde_models::{ACData, CustomError, Example, Light};
use crate::Data;
use actix_web::http::StatusCode;
use actix_web::{get, patch, web, Error, HttpRequest, HttpResponse};
use serde::Deserialize;
use serde_json::Value;

#[get("/")]
pub async fn root() -> HttpResponse {
    HttpResponse::build(StatusCode::OK)
        .content_type("text/html; charset=utf-8")
        .body(include_str!("../../static/root.html"))
}

#[get("/restart")]
pub async fn restart(req: HttpRequest, _: Authorized) -> Result<web::Json<Example>, Error> {
    match req.app_data::<Data<InterraTcpClient>>() {
        Some(interra) => interra.reconnect().await?,
        None => return Err(CustomError::internal_server_error("couldn't restart tcp client") ),
    }

    Ok(web::Json(Example {
        message: "restarted!".to_string(),
    }))
}

#[get("/lights")]
pub async fn get_lights(req: HttpRequest, _: Authorized) -> Result<web::Json<Vec<Light>>, Error> {
    match req.app_data::<Data<InterraTcpClient>>() {
        Some(interra) => Ok(web::Json(interra.get_room_lights(12).await?)),
        None => Err(CustomError::internal_server_error(
            "tcp client suffering, sorry!",
        )),
    }
}

#[get("/lights/{id}")]
pub async fn get_light(req: HttpRequest, _: Authorized) -> Result<web::Json<Light>, Error> {
    let lights = match req.app_data::<Data<InterraTcpClient>>() {
        Some(interra) => interra.get_room_lights(12).await?,
        None => {
            return Err(CustomError::internal_server_error(
                "tcp client suffering, sorry!",
            ))
        }
    };
    match lights
        .into_iter()
        .find(|v| v.id == req.match_info().query("id"))
    {
        Some(light) => Ok(web::Json(light)),
        None => Err(CustomError::bad_request(
            "this is NOT a real ID",
        )),
    }
}
#[patch("/lights/{id}")]
pub async fn set_light(
    req: HttpRequest,
    data: web::Json<Value>,
    _: Authorized,
) -> Result<web::Json<Light>, Error> {
    let id = req.match_info().get("id").ok_or(CustomError::bad_request(
        "this is NOT a real ID",
    ))?;

    let active = match data.get("active") {
        Some(active) => bool::deserialize(active)?,
        None => {
            return Err(CustomError::bad_request(
                "terrible json. I am sorry",
            ))
        }
    };

    let light = Light {
        id: id.to_string(),
        active,
    };

    // let x = req.app_data::<Data>().unwrap();
    match req.app_data::<Data<InterraTcpClient>>() {
        Some(interra) => {
            interra.switch_light(light.id_u16()?, active).await?;
            Ok(web::Json(light))
        }
        None => Err(CustomError::internal_server_error(
            "tcp client suffering, sorry!",
        )),
    }
}

#[get("/ac")]
pub async fn get_ac(req: HttpRequest, _: Authorized) -> Result<web::Json<ACData>, Error> {
    match req.app_data::<Data<InterraTcpClient>>() {
        Some(interra) => Ok(web::Json(interra.get_ac_info(12).await?)),
        None => {
            Err(CustomError::internal_server_error(
                "tcp client suffering, sorry!",
            ))
        }
    }
}
#[patch("/ac")]
pub async fn set_ac(
    req: HttpRequest,
    data: web::Json<ACData>,
    _: Authorized,
) -> Result<web::Json<ACData>, Error> {
    if let Some(t) = data.set_temp {
        if t > 25 {
            return Err(CustomError::bad_request(
                "sorry, 25 is the max temp!",
            ));
        } else if t < 20 {
            return Err(CustomError::bad_request(
                "sorry, 20 is the min temp!",
            ));
        }
    }

    match req.app_data::<Data<InterraTcpClient>>() {
        Some(interra) => Ok(web::Json(interra.set_ac_info_room12(&data).await?)),
        None => Err(CustomError::internal_server_error(
            "tcp client suffering, sorry!",
        )),
    }
}
