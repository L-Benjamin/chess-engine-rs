use actix::{Addr, Message};

use crate::wsclient::WsClient;

// A message used to signify a new connection 
#[derive(Message)]
#[rtype(result = "()")]
pub struct Connect {
    pub addr: Addr<WsClient>,
}

// A message used to signify a disconnection
#[derive(Message)]
#[rtype(result = "()")]
pub struct Disconnect {
    pub addr: Addr<WsClient>,
}

// A message used to represent a client command
#[derive(Message)]
#[rtype(result = "()")]
pub enum ClientDemand {
    Move {
        addr: Addr<WsClient>,
        s: String,
    },
    Play {
        addr: Addr<WsClient>,
    },
    Invite,
}

// A message used to represent a server command
#[derive(Clone, Message)]
#[rtype(result = "()")]
pub enum ClientCommand {
    Info(String),
    State(String),
}