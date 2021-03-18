use std::sync::{Arc, Barrier};

use actix::{Actor, Context, Handler, Message};
use chess::{Game, MoveGenerator};

use crate::threads;

/// Ask the engine to compute a move and give back the result
#[derive(Message)]
#[rtype(result = "String")]
pub struct EngineAskMove;

/// Tell the engine a move has been played
#[derive(Message)]
#[rtype(result = "()")]
pub struct EngineMakeMove {
    pub mv: String,
}

//#################################################################################################
//
//                                     struct Engine
//
//#################################################################################################

pub struct Engine {
    game: Game,
    sync: Arc<Barrier>,
}

// ================================ traits impl

impl Actor for Engine {
    type Context = Context<Self>;
}

impl Default for Engine {
    fn default() -> Engine {
        Engine {
            game: Game::default(),
            sync: threads::start_threads(),
        }
    }
}

impl Handler<EngineAskMove> for Engine {
    type Result = String;

    fn handle(&mut self, msg: EngineAskMove, _: &mut Self::Context) -> String {
        /*if let Some(mv) = threads::launch_search(self.game.clone(), &self.sync) {
            //msg.addr.send(EngineMove {mv: mv.to_string()});
        } 

        "".to_owned()*/

        self.game.legals().next().unwrap().to_string()
    }
}

impl Handler<EngineMakeMove> for Engine {
    type Result = ();

    fn handle(&mut self, msg: EngineMakeMove, _: &mut Self::Context) {
        if let Ok(mv) = self.game.parse_move(&msg.mv) {
            self.game = self.game.do_move(mv);
        }
    }
}
