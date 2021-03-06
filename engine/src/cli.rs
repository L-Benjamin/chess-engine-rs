use std::time::Duration;
use std::{env, io, thread};
use std::io::Write;
use std::str::FromStr;

use anyhow::{Error, Result};
use clap::{App, Arg};

use chess::prelude::*;
use engine::Engine;

/// The default fen used, the starting position.
const DEFAULT_FEN: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

/// The maximum number of moves displayed in move history.
const MAX_HISTORY: usize = 24;

/// The text displayed when the user types "help".
const HELP: &str = r#"Available commands:
  help            : prints this message.
  play <move>     : plays the given <move>, encoded in pure algebraic coordinate notation.
  reset           : resets the game to it's original state.
  think <seconds> : starts the engine for <seconds> seconds.
  do              : plays the engine's preferred move.
  auto <seconds>  : plays the engine against itself, with <seconds> seconds to think for each move.
  exit            : exits the cli."#;

/// The global state of the cli.
struct State {
    engine: Engine,
    buffer: String,
    history: Vec<Move>,
}

// ================================ Utils

impl State {
    /// Print the board and it's fen representation.
    /// Returns true if the game has ended.
    fn print_board(&self) -> bool {
        // Clear the screen.
        print!("\x1B[2J\x1B[1;1H");
        
        // Fen string.
        let board = self.engine.read_board();
        println!("Fen string: \"{}\"", board);

        // Print history.
        if self.history.is_empty() {
            println!("No move history yet.");
        } else {
            print!("Move history: ");

            let history_string = if self.history.len() > MAX_HISTORY {
                print!("..., ");
                &self.history[self.history.len()-MAX_HISTORY..]
            } else {
                &self.history[..]
            }.iter().map(|mv| format!("{}", mv)).collect::<Vec<_>>().join(", ");
                
            println!("{}", history_string);
        }
        
        // Board pretty-print.
        println!("\t\t\t{}", board.pretty_print().replace("\n", "\n\t\t\t"));

        // Formats the game status.
        match board.status() {
            Status::Playing => return false,
            Status::Draw => println!("The game is drawn."),
            Status::Win(color) => match color {
                Color::White => println!("White won the game."),
                Color::Black => println!("Black won the game."),
            }
        }

        true
    }

    /// Print what the engine think is best.
    fn print_engine(&self) {
        if self.engine.read_board().status().is_playing() {
            if let Some(mv) = self.engine.get_best_move() {
                println!("Engine's preferred move: {}.\nFurthest depth searched: {}.", mv, self.engine.get_current_depth());
            } else {
                println!("Engine hasn't had time to think yet.")
            }
        }
    }

    /// Read a line from the terminal.
    fn read_tokens(&mut self) -> Vec<String> {
        // The prompt.
        print!(">>> ");

        io::stdout().flush().ok();
        self.buffer.clear();
        io::stdin().read_line(&mut self.buffer).expect("Cannot read line");

        // Tokenize.
        self.buffer.split_ascii_whitespace().map(|s| s.to_string()).collect()
    }

    /// Ask the user to press enter before continuing.
    fn ask_ok(&mut self) {
        println!("Press enter to continue...");
        io::stdin().read_line(&mut self.buffer).expect("Cannot read line");
    }

    /// Makes the engine think for duration seconds.
    fn think_for(&mut self, duration: Duration) {
        self.engine.start();
        thread::sleep(duration);
        self.engine.stop();
    }

    /// Plays the given move.
    fn play_move(&mut self, mv: Move) {
        let mut board = self.engine.write_board();

        // Sanity check.
        assert!(board.is_pseudo_legal(mv) && board.is_legal(mv), "Tried to play illegal move");

        self.history.push(mv);
        board.do_move(mv);
    }
}

// ================================ Commands

impl State {
    /// Parses and plays the given move.
    fn play(&mut self, args: &mut impl Iterator<Item = String>) -> Result<()> {
        let mv = self.engine.read_board().parse_move(
            &args.next().ok_or(Error::msg("Cannot find <move> argument."))?
        )?;

        self.play_move(mv);

        Ok(())
    }

    /// Reverts the last move.
    fn back(&mut self) -> Result<()> {
        if self.history.is_empty() {
            return Err(Error::msg("No move to undo."));
        }

        let mv = self.history.pop().unwrap();
        let mut board = self.engine.write_board();
        board.undo_move(mv);

        Ok(())
    }

    /// Lets the engine think for a given time, in seconds, parsed from the arguments.
    fn think(&mut self, args: &mut impl Iterator<Item = String>) -> Result<()> {
        if !self.engine.read_board().status().is_playing() {
            return Err(Error::msg("Game has ended. \"undo\" last move or \"reset\" the game."));
        }

        let duration = Duration::from_secs_f64(
            f64::from_str(&args.next().ok_or(Error::msg("Can't find <seconds> argument."))?)?
        );

        self.think_for(duration);

        Ok(())
    }

    /// Performs the engine's preferred move.
    fn do_engine(&mut self) -> Result<()> {
        let mv = self.engine.get_best_move().ok_or(Error::msg("Engine has no move to play yet. Let it \"think\"."))?;
        self.play_move(mv);

        Ok(())
    }

    /// Makes the engine auto-play against itself, with the parsed given time, in seconds, to think between each move.
    fn auto(&mut self, args: &mut impl Iterator<Item = String>) -> Result<()> {
        let duration = Duration::from_secs_f64(
            f64::from_str(&args.next().ok_or(Error::msg("Can't find <seconds> argument."))?)?
        );

        while !self.print_board() {
            // Get the engine's preferred move.
            self.think_for(duration);
            let mv = self.engine.get_best_move().expect("Engine found nothing");
            
            // Play the move.
            self.play_move(mv);
        }

        self.ask_ok();
        Ok(())
    }

    /// Resets the board to it's initial state.
    fn reset(&mut self, fen: &str) -> Result<()> {
        // Reset the board.
        let mut board = self.engine.write_board();
        *board = Board::new(fen).unwrap();

        // Reset the history.
        self.history.clear();

        Ok(())
    }
}

/// The main function parses the programs arguments, initializes the chess library
/// and the engine and then enter a REPL.
fn main() -> Result<()> {
    // Initializes the chess library.
    chess::init();

    // Get the args to the program.
    let args = App::new("Rush Chess Engine CLI")
        .version(env!("CARGO_PKG_VERSION"))
        .author("Benjamin Lefebvre")
        .about("A command line interface for playing the Rush chess engine in the terminal.")
        .arg(Arg::with_name("fen")
            .short("f")
            .long("fen")
            .value_name("FEN")
            .help("Sets the fen string to use as the starting position, the default position is used if no fen string is provided. Use double-quotes.")
            .takes_value(true))
        .get_matches();

    // The fen string used for the position.
    let default_fen = args.value_of("fen").unwrap_or(DEFAULT_FEN);

    // Construct the state.
    let mut state = State {
        // Parse fen and create board, then engine.
        engine: Engine::new(Board::from_str(default_fen)?),
        buffer: String::new(),
        history: Vec::new(),
    };

    // The REPL.
    loop {
        // Print the state of the board and of the engine.
        state.print_board();
        state.print_engine();

        let mut args = state.read_tokens().into_iter();

        if let Some(command) = args.next() {
            let res = match command.as_str() {
                "help" => {
                    println!("{}", HELP);
                    state.ask_ok();
                    Ok(())
                },
                "play" => state.play(&mut args),
                "back" => state.back(),
                "think" => state.think(&mut args),
                "do" => state.do_engine(),
                "auto" => state.auto(&mut args),
                "reset" => state.reset(&default_fen),
                "exit" => {
                    println!("Goodbye.");
                    break;
                },
                unknown => Err(Error::msg(format!("Unknown command: \"{}\". Type \"help\" to get a list of available commands.", unknown))),
            };

            if let Err(e) = res {
                eprintln!("{}", e);
                state.ask_ok();
            }
        }
    }

    // Successfully close the program.
    Ok(())
}