use std::sync::Arc;

use chess::board::Board;
use chess::moves::Move;
use chess::piece::Piece;

use crate::engine::GlobalInfo;
use crate::{eval, utils};
use crate::movepick::MovePicker;
use crate::params;
use crate::table::{TableEntry, TableEntryFlag};

/// A struct holding all the necessary information for a search thread.
#[derive(Debug)]
pub(crate) struct Search {
    info: Arc<GlobalInfo>,
    best_move: Option<Move>,
    depth: u8,
    board: Board,
    buffer: Vec<Move>,
    seed: u32,
}

// ================================ pub(crate) impl

impl Search {
    /// Creates a new search struct, ready to bes used for searching the game tree.
    pub(crate) fn new(info: Arc<GlobalInfo>) -> Search {
        Search {
            info,
            best_move: None,
            depth: 0,
            board: Board::default(),
            buffer: Vec::new(),
            seed: 0,
        }
    }

    /// The loop run by threads
    pub(crate) fn thread_main(&mut self) {
        loop {
            // The start barrier.
            self.info.wait();
    
            // The stop flag was set: we must return from this function. The thread will be joined.
            if self.info.should_stop() {
                return;
            }
    
            // Search the position while the flag is on.
            self.search_position();
    
            // The end search barrier.
            self.info.wait();
        }
    }

}

// ================================ impl

impl Search {
    /// Search the position until told to stop.
    fn search_position(&mut self) {
        // Clone global board and get search depth.
        const MAX_IDX: usize = params::ASPIRATION_WINDOW.len() - 1;
        
        { // Update the board.
            let ply = self.board.get_ply();
            self.board = self.info.board();
            if self.board.get_ply() != ply {
                // New board, reset some fields.
                self.best_move = None;
            }
        }
        
        // Compute first reference score.
        let best_score = self.quiescence(f32::NEG_INFINITY, f32::INFINITY);
        
        'main: loop {
            // Get the depth this thread needs to search to.
            let search_depth = self.info.thread_search_depth();
            
            // Get the first values of alpha and beta in the aspiration window.
            let mut alpha = best_score - params::ASPIRATION_WINDOW[0];
            let mut beta = best_score + params::ASPIRATION_WINDOW[0];
            
            let (mut alpha_idx, mut beta_idx) = (0, 0);
            
            loop {
                let best_score = self.alpha_beta(alpha, beta, true, search_depth, search_depth);
                
                if !self.info.is_searching() {
                    break 'main;
                }
                
                if self.info.search_depth() >= search_depth {
                    break;
                }
                
                if best_score <= alpha {
                    alpha_idx = MAX_IDX.min(alpha_idx + 1);
                    alpha = best_score - params::ASPIRATION_WINDOW[alpha_idx];
                } else if best_score >= beta {
                    beta_idx = MAX_IDX.min(beta_idx + 1);
                    beta = best_score + params::ASPIRATION_WINDOW[beta_idx];
                } else {
                    break;
                }
            }
            
            if let Some(mv) = self.best_move {
                self.info.report_move(mv, search_depth);
            }
        }
    }
    
    /// The alpha-beta negamax algorithm, with a few more heuristics in it.
    pub(crate) fn alpha_beta(&mut self, mut alpha: f32, beta: f32, do_null: bool, mut depth: u8, search_depth: u8) -> f32 {        
        if depth == 0 {
            return self.quiescence(alpha, beta);
        }
        
        if utils::is_pseudo_draw(&self.board, alpha, self.depth == 0) {
            if alpha >= beta {
                return utils::prng_draw_value(&mut self.seed);
            }
        }
        
        if self.depth >= params::MAX_DEPTH {
            return eval::eval(&self.board);
        }
        
        if let Some((mv, score)) = self.info.get_table().probe(self.board.get_zobrist(), alpha, beta, depth) {
            if self.board.is_pseudo_legal(mv) && self.board.is_legal(mv) {
                if score >= alpha && self.depth == 0 {
                    self.best_move = Some(mv);
                }
                return score;
            }
        }
        
        let old_alpha = alpha;
        let in_check = self.board.get_checkers().not_empty();
        
        if in_check {
            depth += 1;
        } else if do_null && self.depth > 0 && depth >= 4 {
            // TODO: Null move heuristic
        }
    
        let mut best_score = f32::NEG_INFINITY;
        let mut best_move = None;
        let mut picker = MovePicker::new(&self.board, &self.buffer);
        let mut move_count = 0;
    
        while let Some(range) = picker.next(&self.board, &mut self.buffer) {
            for i in range {
                let mv = self.buffer[i];

                if !self.board.is_legal(mv) {
                    continue;
                }

                self.depth += 1;
                self.board.do_move(mv);
                let score = -self.alpha_beta(-beta, -alpha, do_null, depth-1, search_depth);
                self.board.undo_move(mv);
                self.depth -= 1;

                if self.info.search_depth() >= search_depth || !self.info.is_searching() {
                    return 0.0;
                }
        
                if score > best_score {
                    best_score = score;
                    best_move = Some(mv);
                    
                    if score > alpha {
                        if score >= beta {
                            if !mv.is_capture() {
                                // TODO: killer heuristic
                            }

                            self.info.get_table().insert(TableEntry::new(
                                &self.board,
                                mv, 
                                beta,
                                depth, 
                                TableEntryFlag::Beta
                            ));
                            
                            return beta;
                        }
        
                        alpha = score;
                    }
                }
                
                move_count += 1;
            }
        }
        
        if move_count == 0 {
            return if in_check {
                -params::value_of(Piece::King) + self.depth as f32
            } else {
                0.0
            };
        }
        
        if alpha != old_alpha {
            self.info.get_table().insert(TableEntry::new(
                &self.board,
                best_move.unwrap(), 
                best_score, 
                depth, 
                TableEntryFlag::Exact
            ));
            
            if self.depth == 0 {
                self.best_move = best_move;
            }
        } else {
            self.info.get_table().insert(TableEntry::new(
                &self.board,
                best_move.unwrap(), 
                best_score, 
                depth, 
                TableEntryFlag::Alpha
            ));
        }
        
        alpha
    }

    /// Return the value of the position, computed with a quiescent search (only considering captures).
    fn quiescence(&mut self, mut alpha: f32, beta: f32) -> f32 {
        if utils::is_pseudo_draw(&self.board, alpha, self.depth == 0) {
            if alpha >= beta {
                return utils::prng_draw_value(&mut self.seed);
            }
        }
        
        let stand_pat = eval::eval(&self.board);
    
        if self.depth >= params::MAX_DEPTH {
            return stand_pat;
        }
    
        if stand_pat >= beta {
            return beta;
        }
    
        let mut big_delta = params::value_of(Piece::Queen);
        if utils::may_promote(&self.board) {
            big_delta += params::value_of(Piece::Queen) - params::value_of(Piece::Pawn);
        }
    
        if stand_pat < alpha - big_delta {
            return alpha;
        }
    
        alpha = alpha.max(stand_pat);
    
        let mut picker = MovePicker::new(&self.board, &self.buffer);
    
        'search: while let Some(range) = picker.next(&self.board, &mut self.buffer) {
            for i in range {
                let mv = self.buffer[i];

                if mv.is_quiet() {
                    break 'search; // TODO: Better movepicker
                }

                if !mv.is_capture() || params::value_of(mv.get_capture()) + params::DELTA < alpha || !self.board.is_legal(mv) {
                    continue;
                }
        
                self.depth += 1;
                self.board.do_move(mv);
                let score = -self.quiescence(-beta, -alpha);
                self.board.undo_move(mv);
                self.depth -= 1;
        
                if !self.info.is_searching() {
                    return 0.0;
                }
        
                if score > alpha {
                    if score >= beta {
                        return beta;
                    }
                    alpha = score;
                }
            }
        }
        
        alpha
    }
}
