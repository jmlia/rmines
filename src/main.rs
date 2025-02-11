//

use rand::Rng;
use std::{io::{self, Write}, time::SystemTime};

mod game;
use game::*;

enum ParseResult<'a> {
    Ok,
    TooManyArguments,
    MissingArgument,
    InvalidArgument(&'a str)
}

fn parse_arguments<'a>(line: &'a str, args: &mut [usize], mandatory: bool) -> ParseResult<'a> {

    let mut args_it = args.iter_mut();
    
    for slice in line.split(',') {

        let Some(arg) = args_it.next() else {
            return ParseResult::TooManyArguments };

        match slice.parse::<usize>() {
            Ok(n) if n > 0 => *arg = n,
            // If `slice' is empty but the argument is mandatory, report the case as an error.
            Err(error) if *error.kind() == std::num::IntErrorKind::Empty => {
                if mandatory {
                    return ParseResult::MissingArgument;
                }
            },
            _ => return ParseResult::InvalidArgument(slice),
        }
    }

    ParseResult::Ok
} 

fn main() {

    // Welcome message.
    println!("\nWelcome to rmines!\n\
             A default board of 10x10 cells and approximately 50 mines has been crated.\n\
             To start a new game with a different board, just type in the command 'n <rows>, <cols>, \
             <mines>'\nType in 'h' or '?' at the prompt to list all the commands available.\n\
             Have fun!\n");

    let prefix: &'static str = ">>";
    let mut board = Board::new(10, 10, 50).unwrap();
    let mut line = String::new();
    let mut rng = rand::thread_rng();    
    let mut start_time = SystemTime::now();
    
    'main:
    loop {

        // Compute the total playing time.
        let mut playing_time: String =
            String::from("(Could not compute the total playing time.)");

        if let Ok(duration) = start_time.elapsed() {
            let seconds_elapsed = duration.as_secs();
            playing_time = format!("{}h {}m {}s",
                                   seconds_elapsed/3600,
                                   (seconds_elapsed % 3600)/60,
                                   ((seconds_elapsed % 3600) % 60));
        }

        // Print the board and other information related to the current game.
        print!("{}\n\
                Located {flagged} of {mine_count} mines\n\
                Total playing time: {playing_time}\n\n\
                {prefix} ",
               board, flagged = board.get_flagged_count(),
               mine_count = board.get_mine_count());
            io::stdout().flush().unwrap();
        
        line.clear();
        match io::stdin().read_line(&mut line) {

            Ok(_) => {

                // Eat up all whitespace before processing the input line.
                line.retain(|c| !c.is_whitespace());

                if let Some(cmd) = line.chars().next() {

                    // Arguments are mandatory for 'f/>' but optional for 'n' and 'x'. If not given,
                    // any missing argument is replaced by a random value chosen appropriately. All
                    // arguments must be convertible to `usize'.

                    let arg_line = line.strip_prefix(cmd).unwrap();

                    match cmd {
                        'n' => { // Start a new game.

                            // Default arguments (make a board no larger than the current one).
                            let mut args: [usize; 3] = [
                                rng.gen_range(1..=board.get_rows()), // Number of rows
                                rng.gen_range(1..=board.get_cols()), // Number of columns
                                0
                            ];

                            // Default number of mines. // CHECK THIS
                            args[2] = rng.gen_range(1..(args[0] * args[1]));

                            match parse_arguments(arg_line, &mut args, false) {
                                ParseResult::TooManyArguments => {
                                    println!("{prefix} '{cmd}': too many arguments, expected three \
                                              at most: `[rows]', `[columns]', and `[mine count]'.\n");
                                    continue;
                                }
                                ParseResult::InvalidArgument(slice) => {
                                    println!("{prefix} '{cmd}':  '{slice}' is not a valid coordinate.\n");
                                    continue;
                                },
                                _ => {}
                            }

                            // Try to create a new board.
                            match Board::new(args[0], args[1], args[2]) {
                                Ok(new_board) => {
                                    println!("{prefix} Starting a new game. The new board has {rows} rows, \
                                              {cols} columns, and (approximately) {count} mines.\n",
                                             rows = args[0], cols = args[1], count = args[2]);
                                    board = new_board;
                                    start_time = SystemTime::now();
                                },
                                Err(BoardError::NullArea) => {
                                    println!("{prefix} '{cmd}': Cannot create a board with zero rows or columns!\n");
                                },
                                Err(BoardError::TooManyMines) => {
                                    println!("{prefix} '{cmd}': Too many mines for such a small board!\n");
                                }
                            }
                        },

                        'x' => { // Explore the cell at the given coordinate.
                            
                            // Randomly choose a cell to explore if the user doesn't provide any.
                            let mut args: [usize; 2] = [
                                rng.gen_range(1..=board.get_rows()), // Row
                                rng.gen_range(1..=board.get_cols()), // Column
                            ];

                            // Parse arguments.
                            match parse_arguments(arg_line, &mut args, false) {
                                ParseResult::TooManyArguments => {
                                    println!("{prefix} '{cmd}': too many arguments, expected \
                                              two at most: `[row]', `[colum]'.\n");
                                    continue;
                                },
                                ParseResult::InvalidArgument(slice) => {
                                    println!("{prefix} '{cmd}': '{slice}' is not a valid coordinate.\n");
                                    continue;
                                },
                                _ => {}
                            }
                            
                            // Try to add the specified coordinate to the unexplored cache.
                            match board.cache((args[0], args[1])) {
                                CacheResult::InvalidCoordinate => {
                                    println!("{prefix} '{cmd}': invalid cell coordinate ({x}, {y}).\n",
                                             x = args[0], y = args[1]);
                                    continue 'main;
                                },
                                CacheResult::Clear => {
                                    println!("{prefix} '{cmd}': the cell at ({x}, {y}) is clear.\n",
                                             x = args[0], y = args[1]);
                                    continue 'main;
                                },
                                CacheResult::Ok => {

                                    loop {

                                        // Explores the board greedily, that is, it keeps exploring clear
                                        // neighborhoods until it runs into one that is mined.

                                        match board.explore() {
                                            ExploreResult::Ok => {}, // Added for readability.
                                            ExploreResult::ClearBoard => {
                                                println!("{prefix} Congratulations! All mines have been found!\n\n\
                                                          {board}\n");
                                                break 'main;
                                            },
                                            ExploreResult::EmptyCache => break,
                                            ExploreResult::Mined => {
                                                println!("{prefix} The cell is mined!\n\n\
                                                          {board}\n\
                                                          Game over!\n");
                                                // TODO: ask the user if they want to start a new game.
                                                break 'main;
                                            },
                                        }
                                    }
                                }
                            }
                        },

                        'f' | '>' => { // Flag the cell at the coordinate given.

                            let mut args: [usize; 2] = [ 0; 2 ];

                            match parse_arguments(arg_line, &mut args, true) {
                                ParseResult::MissingArgument => {
                                    println!("{prefix} '{cmd}': too few arguments passed in.\n");
                                    continue;
                                },
                                ParseResult::TooManyArguments => {
                                    println!("{prefix} '{cmd}': too many arguments, expected \
                                              two at most: `[row]', `[colum]'.\n");
                                    continue;
                                },
                                ParseResult::InvalidArgument(slice) => {
                                    println!("{prefix} '{cmd}': '{slice}' is not a valid coordinate.\n");
                                    continue;
                                },
                                _ => {}
                            }

                            if !board.toggle_flag_at((args[0], args[1])) {
                                println!("{prefix} '{cmd}': ");
                                continue 'main;
                            }
                        },

                        'h' | '?' =>  { // Print the list of available commands.

                            if !arg_line.is_empty() {
                                println!("{prefix} '{cmd}': unknown command. Did you mean 'h'?\n");
                                continue;
                            }
                            
                            println!("\nAvailable commands:\n\n\
                                      - n   rows, columns, mines  start a new game with the given board dimensions and mines.\n\
                                      - x   row, col              explore the cell at (row, col).\n\
                                      - f/> row, col              flag the cell at (row, col).\n\
                                      - h                         print this message.\n\
                                      - q                         quit the game.\n\n\
                                      Arguments to the `n' and `x' command are optional.\n\
                                      An appropriate value will be chosen at random for each missing argument.\n");
                            continue;
                        },

                        'q' => { // Quit the game.
                            if !arg_line.is_empty() {
                                println!("{prefix} '{cmd}': unknown command. Did you mean 'q'?\n");
                                continue;
                            }

                            println!("Goodbye!");
                            break;
                        },

                        _ => { // Unknown command passed in.
                            println!("{prefix} Unknown commmand '{cmd}'.\n");
                        },
                    }
                }
            },

            Err(_) => {
                println!("{prefix} Error while reading input. Quitting the game...");
                break;
            }
        }
    }
}
