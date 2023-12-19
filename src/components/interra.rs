use crate::components::serde_models::{ACData, ACDatum, FanSpeed, Light};
use serde_json::Value;
use std::io::ErrorKind;
use std::time::Duration;
use std::{env, io};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader, BufWriter, Result};
use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};
use tokio::net::TcpStream;
use tokio::sync::{Mutex, RwLock};
use tokio::time;

pub struct InterraTcpClient {
    sink: Mutex<BufWriter<OwnedWriteHalf>>,
    stream: Mutex<BufReader<OwnedReadHalf>>,
    token: RwLock<String>,
}

impl InterraTcpClient {
    async fn establish() -> Result<(BufWriter<OwnedWriteHalf>, BufReader<OwnedReadHalf>, String)> {
        let ip = env::var("TCP_IP")
            .map_err(|_| io::Error::new(ErrorKind::Other, "TCP_IP not supplied in .env"))?;
        let port = env::var("PORT")
            .map_err(|_| io::Error::new(ErrorKind::Other, "PORT not supplied in .env"))?
            .parse::<u16>().map_err(|_| io::Error::new(ErrorKind::Other, "this is not a port"))?;
        let username = env::var("USERNAME")
            .map_err(|_| io::Error::new(ErrorKind::Other, "USERNAME not supplied in .env"))?;
        let password = env::var("PASSWORD")
            .map_err(|_| io::Error::new(ErrorKind::Other, "PASSWORD not supplied in .env"))?;

        let (read, write) = TcpStream::connect((ip, port)).await?.into_split();

        let mut reader = BufReader::new(read);
        let mut writer = BufWriter::new(write);

        println!("Authenticating...");

        let payload = format!("{{'data':{{'userName':'{username}','password':'{password}'}},'meta':{{'authID':null,'content_type':null,'error':null,'errorCode':null,'flags':null,'requestType':500,'scheme':null,'serverDateTime':null,'server_version':null,'version':null}}}}\n");

        writer.write(payload.as_bytes()).await?;
        writer.flush().await?;

        let mut token = String::new();
        reader.read_line(&mut token).await?;
        println!("TCP Listener >> {}", token.trim());
        token = serde_json::from_str::<Value>(&token)?["meta"]["authID"].to_string();

        println!("Connected.");

        Ok((writer, reader, token))
    }

    pub async fn connect() -> Result<Self> {
        println!("Connecting to Interra...");
        let (w, r, token) = Self::establish().await?;

        Ok(Self {
            sink: Mutex::new(w),
            stream: Mutex::new(r),
            token: RwLock::new(token),
        })
    }

    pub async fn reconnect(&self) -> Result<()> {
        println!("Reconnecting...");
        let (w, r, token) = Self::establish().await?;

        *self.sink.lock().await = w;
        *self.stream.lock().await = r;
        *self.token.write().await = token;

        Ok(())
    }

    // todo rm maybe
    pub async fn keep_alive(&self) -> Result<()> {
        // PREVENT R/W ATTEMPTS WHILE KEEPALIVE IN PROGRESS
        let mut lock_sink = self.sink.lock().await;
        let mut lock_stream = self.stream.lock().await;

        lock_sink.write_all(b"{}\n").await?;
        if let Err(_) = lock_sink.flush().await {
            println!("KeepAlive sink failed, restarting TCP connection...");
            drop(lock_stream);
            drop(lock_sink);

            return self.reconnect().await;
        }

        // loop until you get the "response" instead of {"data":{"readValue":"1","isActive":true,"id":108},"meta":{"requestType":19}}
        // todo find a way to stop it but use this solution for now
        let mut out = String::new();
        let mut attempt = 1;

        loop {
            println!("KeepAlive attempt ({attempt}) in progress...");
            lock_stream.read_line(&mut out).await?;

            let json = serde_json::from_str::<Value>(&out);
            match json {
                Ok(value) if value.get("data").is_none() => {
                    println!("KeepAlive successful with TCP output >> {out}");
                    break;
                }
                _ => {
                    println!("KeepAlive failed with TCP output >> {out}\nCollecting next line in stream...");
                    out.clear();
                    attempt += 1;
                }
            };

            // unlikely but just to be sure
            if attempt > 5 {
                println!("KeepAlive attempt limit reached, restarting TCP connection...");
                drop(lock_stream);
                drop(lock_sink);

                self.reconnect().await?;
                break;
            }
        }
        Ok(())
    }

    pub async fn read_line(&self) -> Result<Value> {
        let mut out = String::new();
        let mut lock = self.stream.lock().await;

        loop {
            let byte = lock.read_line(&mut out).await?;
            println!("TCP Listener ({byte}) >> {out}");

            let json = serde_json::from_str::<Value>(&out)?;

            match json.pointer("/data/id").map(|v| v.as_u64()).flatten() {
                Some(v) if v == 108 => {
                    continue;
                }
                _ => return Ok(json),
            }
        }
    }

    pub async fn request(
        &self,
        data: Option<&str>,
        request_type: u8,
        flags: Option<&str>,
    ) -> Result<()> {
        let mut lock = self.sink.lock().await;

        // let readable = lock.get_ref().ready(Interest::READABLE | Interest::WRITABLE).await?.is_readable();
        // if !readable {
        //     drop(lock);  // todo not dropping WILL deadlock execution, fix later for better design
        //     self.reconnect().await?;
        //     lock = self.sink.lock().await;
        // }

        let out = format!(
            "{{'data':{},'meta':{{'authID':{},'content_type':null,'error':null,'errorCode':null,'flags':{},'requestType':{request_type},'scheme':null,'serverDateTime':null,'server_version':null,'version':null}}}}\n",
            data.unwrap_or("null"),
            self.token.read().await,
            flags.unwrap_or("null")
        );
        lock.write_all(out.as_bytes()).await?;

        println!("TCP Listener () << {out}");

        lock.flush().await
    }
    pub async fn request_read(
        &self,
        data: Option<&str>,
        request_type: u8,
        flags: Option<&str>,
    ) -> Result<String> {
        self.request(data, request_type, flags).await?;
        Ok(self.read_line().await?["data"].to_string())
    }

    // actual commands start here
    pub async fn switch_light(&self, id: u16, enable: bool) -> Result<()> {
        self.request(
            Some(&format!(
                "{{'actionType':'{}','id':'{id}','url':null,'value':'0'}}",
                if enable { 1 } else { 2 }
            )),
            14,
            None,
        )
        .await?;
        Ok(())
    }

    pub async fn get_room_lights(&self, room_id: u16) -> Result<Vec<Light>> {
        let response = self
            .request_read(
                Some(&format!(
                    "{{'id':'{room_id}','objectType':'{}'}}",
                    DeviceType::Lights.id()
                )),
                20,
                None,
            )
            .await?;
        let lights: Vec<Light> = serde_json::from_str(&response)?;
        Ok(lights)
    }

    pub async fn get_ac_info(&self, room_id: u16) -> Result<ACData> {
        let response = self
            .request_read(
                Some(&format!(
                    "{{'id':'{room_id}','objectType':'{}'}}",
                    DeviceType::Ac.id()
                )),
                20,
                None,
            )
            .await?;
        let ac: Vec<ACDatum> = serde_json::from_str(&response)?;
        Ok(ACData::from(ac))
    }

    // hardcoded my room
    pub async fn set_ac_info_room12(&self, ac: &ACData) -> Result<ACData> {
        let mut ac_old = self.get_ac_info(12).await?;

        if let (Some(t), Some(t_old)) = (ac.set_temp, ac_old.set_temp) {
            ac_old.set_temp = Some(t);
            let increase_temp = t as i8 - t_old as i8;

            if increase_temp > 0 {
                for _ in 0..increase_temp {
                    self.request(
                        Some("{'actionType':13,'id':'64','url':null,'value':'0'}"),
                        14,
                        None,
                    )
                    .await?;
                    time::sleep(Duration::from_millis(300)).await;
                }
            } else if increase_temp < 0 {
                for _ in 0..increase_temp.abs() {
                    self.request(
                        Some("{'actionType':13,'id':'63','url':null,'value':'0'}"),
                        14,
                        None,
                    )
                    .await?;
                    time::sleep(Duration::from_millis(300)).await;
                }
            }

            time::sleep(Duration::from_millis(300)).await;
        }

        if let Some(f) = ac.fan_speed {
            ac_old.fan_speed = Some(f);
            let id: u8 = match f {
                FanSpeed::Auto => 66,
                FanSpeed::Slow => 67,
                FanSpeed::Medium => 68,
                FanSpeed::Fast => 69,
            };

            self.request(
                Some(&format!(
                    "{{'actionType':13,'id':'{id}','url':null,'value':'0'}}"
                )),
                14,
                None,
            )
            .await?;

            time::sleep(Duration::from_millis(300)).await;
        }

        if let Some(a) = ac.active {
            ac_old.active = Some(a);
            let id: u8 = match a {
                true => 57,
                false => 58,
            };

            self.request(
                Some(&format!(
                    "{{'actionType':13,'id':'{id}','url':null,'value':'0'}}"
                )),
                14,
                None,
            )
            .await?;

            time::sleep(Duration::from_millis(300)).await;
        }

        Ok(ac_old)
    }
}

pub enum DeviceType {
    Ac,
    Lights,
}
impl DeviceType {
    fn id(&self) -> u8 {
        match self {
            Self::Ac => 4,
            Self::Lights => 1,
        }
    }
}
