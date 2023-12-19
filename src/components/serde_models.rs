use serde::{Deserialize, Deserializer, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};

#[derive(Serialize)]
pub struct Example {
    pub(crate) message: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CustomError {
    pub message: String,
}
impl CustomError {
    pub fn internal_server_error(message: &str) -> actix_web::Error {
        actix_web::error::ErrorInternalServerError(
            serde_json::to_string(&Self {
                message: message.to_string(),
            })
            .unwrap(),
        )
    }

    pub fn bad_request(message: &str) -> actix_web::Error {
        actix_web::error::ErrorBadRequest(
            serde_json::to_string(&Self {
                message: message.to_string(),
            })
            .unwrap(),
        )
    }

    pub fn unauthorized(message: &str) -> actix_web::Error {
        actix_web::error::ErrorUnauthorized(
            serde_json::to_string(&Self {
                message: message.to_string(),
            })
            .unwrap(),
        )
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Light {
    #[serde(deserialize_with = "light_deser")]
    pub id: String,
    #[serde(rename(deserialize = "isActive"))]
    pub active: bool,
}

impl Light {
    pub fn id_u16(&self) -> Result<u16, actix_web::Error> {
        match &*self.id {
            "ceilingLights" => Ok(13),
            "shelfLight" => Ok(146),
            _ => Err(CustomError::bad_request("this is NOT a real ID",
            )),
        }
    }
}
fn light_deser<'de, D: Deserializer<'de>>(deserializer: D) -> Result<String, D::Error> {
    let id = u16::deserialize(deserializer)?;
    Ok(String::from(match id {
        13 => "ceilingLights",
        146 => "shelfLight",
        _ => "???",
    }))
}

#[derive(Serialize_repr, Deserialize_repr, Debug, Copy, Clone)]
#[repr(u8)]
pub enum FanSpeed {
    Auto = 0,
    Slow = 1,
    Medium = 2,
    Fast = 3,
}
impl FanSpeed {
    fn from(value: &str) -> Option<Self> {
        match value {
            "00" => Some(FanSpeed::Auto),
            "01" => Some(FanSpeed::Slow),
            "02" => Some(FanSpeed::Medium),
            "03" => Some(FanSpeed::Fast),
            _ => None,
        }
    }
}

#[derive(Deserialize)]
pub struct ACDatum {
    id: u8,
    #[serde(rename = "isActive")]
    active: bool,
    #[serde(rename = "readValue", default)]
    value: String,
}

#[derive(Serialize, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ACData {
    pub room_temp: Option<f64>,
    pub set_temp: Option<u8>,
    pub fan_speed: Option<FanSpeed>,
    pub active: Option<bool>,
}
impl From<Vec<ACDatum>> for ACData {
    fn from(ac: Vec<ACDatum>) -> Self {
        let room_temp = ac
            .iter()
            .find(|v| v.id == 60)
            .map(|v| v.value.parse::<f64>().ok())
            .flatten();
        let set_temp = ac
            .iter()
            .find(|v| v.id == 62)
            .map(|v| v.value.parse::<f64>().map(|v| v as u8).ok())
            .flatten();
        let fan_speed = ac
            .iter()
            .find(|v| v.id == 67)
            .map(|v| FanSpeed::from(&v.value))
            .flatten();
        let active = ac.iter().find(|v| v.id == 57).map(|v| v.active);

        Self {
            room_temp,
            set_temp,
            fan_speed,
            active,
        }
    }
}
