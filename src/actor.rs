use std::{thread, time};
use actix::prelude::*;

use StringError;

pub struct EchoActor {
    pub name: String,
}

impl EchoActor {

    pub fn new(name: &str) -> Self {
        Self {
            name: String::from(name),
        }
    }
}

impl Actor for EchoActor {
    type Context = Context<Self>;

    fn started(&mut self, _: &mut Context<Self>) {
        info!("EchoActor is alive.");
    }
    fn stopped(&mut self, _: &mut Context<Self>) {
        info!("EchoActor is stopped.");
    }
}

impl Handler<Greeting> for EchoActor {
    type Result = Result<String, StringError>;

    fn handle(&mut self, msg: Greeting, _: &mut Context<Self>) -> Self::Result {
        thread::sleep(time::Duration::from_millis(100));
        Ok(format!("{}: {}", self.name, msg.message))
    }
}

pub struct Greeting {
    pub message: String,
}
impl Message for Greeting {
    type Result = Result<String, StringError>;
}
