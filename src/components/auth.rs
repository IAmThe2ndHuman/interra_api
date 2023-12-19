use crate::components::serde_models::CustomError;
use actix_web::dev::Payload;
use actix_web::{Error, FromRequest, HttpRequest};
use std::env;
use std::future::{ready, Ready};

pub struct Authorized;

impl FromRequest for Authorized {
    type Error = Error;
    type Future = Ready<Result<Self, Error>>;

    fn from_request(req: &HttpRequest, _: &mut Payload) -> Self::Future {
        let token = req
            .headers()
            .get("Authorization")
            .map(|v| v.to_str().unwrap_or_default());

        // Okay, this looks weird because I wanted to trick my friend into putting in the wrong
        // bearer token for funsies
        // The docs say the token is "not what you think it is"
        // so if they put in "what you think it is" or "not what you think it is" it will reject them
        // with a funny message
        let env_token =
            env::var("AUTH_TOKEN").unwrap_or_else(|_| "backup token thingy".to_string());
        let out = match token {
            Some(out) if env_token == out => Ok(Authorized),
            Some("not what you think it is") | Some("what you think it is") => {
                Err(CustomError::unauthorized("nice try"))
            }
            _ => Err(CustomError::unauthorized("who are you")),
        };

        ready(out)
    }
}
