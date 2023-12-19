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
    // unwrapped because these WONT fail no matter what lol
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
    // #[serde(deserialize_with = "light_description", rename(deserialize = "id"))]
    // pub description: String,
    #[serde(rename(deserialize = "isActive"))]
    pub active: bool,
}

impl Light {
    pub fn id_u16(&self) -> Result<u16, actix_web::Error> {
        match &*self.id {
            "ceilingLights" => Ok(13),
            "shelfLight" => Ok(146),
            _ => Err(CustomError::bad_request(
                "blud really thought bro could jus put any ol \
             id here 💀💀💀 i only accept ceilingLights n shelfLight u from ohio or what",
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
// fn light_ser<'de, S: Serializer<'de>>(id: &str, deserializer: S) -> Result<S::Ok, S::Error> {
//     deserializer.serialize_u16(match id {
//         "ceilingLights" => 13,
//         "shelfLight" => 146,
//         _ => 0
//     })
// }

// #[derive(Serialize, Deserialize, Debug)]
// #[serde(rename = "id")]
// pub enum LightType {
//     #[serde(rename(deserialize = "13"))]
//     Ceiling,
//     #[serde(rename(deserialize = "146"))]
//     Shelf,
//     #[serde(other)]
//     Unknown
// }
// impl LightType {
//     fn id(&self) -> u16 {
//         match self {
//             LightType::Ceiling => 13,
//             LightType::Shelf => 146,
//             LightType::Unknown => 0
//         }
//     }
//     fn description(&self) -> &'static str {
//         match self {
//             LightType::Ceiling => "the two big lights on the ceiling",
//             LightType::Shelf => "my shelf light",
//             LightType::Unknown => "???"
//         }
//     }
// }
//

// #[derive(Serialize)]
// pub struct ACDatum {
//     pub room_temp: f64,
//     pub set_temp: f64,
//     pub fan_speed: u8,
//     pub active: bool
// }
// #[derive(Deserialize, Debug)]
// pub struct ACDatum {
//
//     pub id: ACDatumID,
//     #[serde(rename(deserialize = "isActive"))]
//     pub active: bool,
//     #[serde(rename(deserialize = "readValue"), default)]
//     pub value: String
// }
// #[derive(Serialize, Debug)]
// // #[serde(tag = "id")]
// pub enum ACDatum {
//     Active(bool),
//     FanSpeed(FanSpeed),
//     SetTemp(f64),  // todo change to f64
//     RoomTemp(f64),
//     Unknown
// }
// // WHY WONT SERDE LET ME #[serde(rename = 60)] IT WOULD'VE BEEN SO MUCH EASIE
// impl<'de> Deserialize<'de> for ACDatum {
//     fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: Deserializer<'de> {
//         #[derive(Deserialize)]
//         struct ACDatumContent {
//             id: u8,
//             #[serde(rename = "isActive")]
//             active: bool,
//             #[serde(rename = "readValue", default)]
//             value: String
//         }
//         let ac = ACDatumContent::deserialize(deserializer)?;
//
//         Ok(match ac.id {
//             60 => ACDatum::RoomTemp(ac.value.parse::<f64>().unwrap_or_default()),
//             62 => ACDatum::SetTemp(ac.value.parse::<f64>().unwrap_or_default()),
//             67 => ACDatum::FanSpeed(FanSpeed::from(ac.value)),
//             57 => ACDatum::Active(ac.active),
//             _ => ACDatum::Unknown
//         })
//         // Ok(ACDatumID::Unknown)
//     }
// }
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
// impl TryFrom<Vec<ACDatum>> for ACData{
//     type Error = &'static str;
//
//     fn try_from(value: Vec<ACDatum>) -> Result<Self, Self::Error> {
//         let mut iter = value.into_iter();
//
//         let room_temp = iter.find(|v| v.id == ACDatumID::RoomTemp).ok_or_else(|| "couldn't get room temp")?.value;
//         let set_temp = iter.find(|v| v.id == ACDatumID::SetTemp).ok_or_else(|| "couldn't get set temp")?.value;
//         let fan_speed = iter.find(|v| v.id == ACDatumID::FanSpeed).ok_or_else(|| "couldn't get fan speed")?.value;
//         let active = iter.find(|v| v.id == ACDatumID::Active).ok_or_else(|| "couldn't get active")?.active;
//
//         Ok(Self {
//             active,
//             room_temp: room_temp.parse::<f64>()?,
//             fan_speed:
//
//         })
//     }
// }
