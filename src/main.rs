use rand::{Rng, distributions::{Distribution, Uniform}};
use std::{collections::{HashMap, HashSet}, fmt};
use std::io::{self, Write};

type Coord = (usize, usize);

#[derive(Debug)]
enum BoardError {
    NullArea,
    TooManyMines,
}

enum ExploreResult {
    EmptyCache,
    Mined,
    MinedNeighbourhood,
    Clear,
}

struct Board {
    // Dimensions of the board.
    rows: usize,
    cols: usize,
    area: usize,

    // Number of cells marked as mined.
    flagged: usize,

    // Sets containing the coordinates of the cells containing
    // mines, those pending to be explored and those already explored.
    unexplored: HashSet<Coord>,
    explored: HashSet<Coord>,
    mines_at: HashSet<Coord>,

    // Data related to printing the board to stdout.
    board_string: String,
    labels: HashMap<Coord, usize>,
}

impl fmt::Display for Board {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.board_string)
    }
}

impl Board {

    fn new(rows: usize, cols: usize, mine_count: usize) -> Result<Self, BoardError> {

        let board_area = rows * cols;

        if board_area == 0 {
            return Err(BoardError::NullArea);
        }

        if board_area <= mine_count {
            return Err(BoardError::TooManyMines);
        }

        let row_label_width = rows.to_string().len() + 1;
        let col_label_width = cols.to_string().len() + 2;

        let mut board_string: String =
            String::with_capacity(((col_label_width + 2) * cols + (row_label_width + 2)) * rows);

        // Header
        board_string.push_str(
            format!("{:width$}|", 1, width = (row_label_width + 1) + col_label_width)
                .as_str()
        );

        for col in 2..(cols + 1) {
            board_string.push_str(format!("{:col_label_width$}|", col).as_str());
        }

        board_string.push('\n');

        // One line for each row.
        for row in 0..rows {
            board_string.push_str(format!("{:row_label_width$}|", row + 1).as_str());
            board_string.push_str(
                format!("{:>width$}", '.', width = col_label_width + 1)
                    .repeat(cols)
                    .as_str(),
            );
            board_string.push('\n');
        }

        /* Mine the board by randomly placing (approximately)
         * `mine_count' mines. Note that it is possible that the
         * actual number of mines is less than `mine_count' as the
         * same point may be drawn from the distribution more than
         * once. TODO: handle this case.
         */

        let mut rng = rand::thread_rng();
        let uniform: Uniform<usize> = Uniform::new(0, board_area);

        let mines_at: HashSet<Coord> = uniform
            .sample_iter(&mut rng)
            .take(mine_count)
            .map(|index| (index/cols, index%cols))
            .collect::<HashSet<Coord>>();

        Ok(Board {
            rows,
            cols,
            area: board_area,
            flagged: 0,
            unexplored: HashSet::with_capacity(board_area - mines_at.len()),
            explored: HashSet::with_capacity(board_area - mines_at.len()),
            mines_at,
            labels: board_string
                .match_indices('.')
                .enumerate()
                .map(|(n, (index, _))| ((n/cols, n % cols), index))
                .collect::<HashMap<Coord, usize>>(),
            board_string,
        })
    }

    fn add_to_unexplored(&mut self, coord: Coord) -> bool {

        if coord.0 * self.cols + coord.1 < self.area {
            self.unexplored.insert(coord);
            return true;
        }
        false
    }

    fn reveal_mines(&mut self) {
        for coord in &self.mines_at {
            let index = self.labels.get(coord).unwrap();
            self.board_string.replace_range(*index..(*index + 1), "*");
        }
    }

    fn update_label(&mut self, at: Coord, label: char) {
        let index: usize = *self.labels.get(&at).unwrap();
        let mut buffer: [u8; 2] = [0; 2];
        self.board_string.replace_range(index..(index + 1),
                                        label.encode_utf8(&mut buffer));
    }

    fn get_rows(&self) -> usize {
        self.rows
    }

    fn get_cols(&self) -> usize {
        self.cols
    }

    fn get_mine_count(&self) -> usize {
        self.mines_at.len()
    }

    fn get_flagged_count(&self) -> usize {
        self.flagged
    }

    fn all_mines_located(&self) -> bool {
        self.area - self.explored.len() == self.mines_at.len()
    }
    
    fn toggle_flag_at(&mut self, at: Coord) -> bool {

        if let Some(index) = self.labels.get(&at) {
        
             // Do nothing if the parcel has been explored.
            if !self.explored.contains(&at) {
                if self.board_string.chars().nth(*index).unwrap() == '>' {
                    self.flagged -= 1;
                    self.update_label(at, '.');
                }
                else {
                    self.flagged += 1;
                    self.update_label(at, '>');
                }
            }

            return true;
        }

        false
    }

    fn explore(&mut self) -> ExploreResult {

        if self.unexplored.is_empty() {
            return ExploreResult::EmptyCache;
        }

        // Remove the coordinate from the `unexplored' cache...
        let (row, col) = *self.unexplored.iter().next().unwrap();
        self.unexplored.remove(&(row, col));

        // ...and insert it into the `explored' set.
        if !self.explored.insert((row, col)) {
            return ExploreResult::EmptyCache;
        }

        // 1. The cell is mined!
        if self.mines_at.contains(&(row, col)) {
            self.reveal_mines();
            return ExploreResult::Mined;
        }

        // 2. Check if the neighbourhood is mined.
        let above = row.checked_sub(1);
        let below = if row < self.rows - 1 { Some(row + 1) } else { None };
        
        let left = col.checked_sub(1);
        let right = if col < self.cols - 1 { Some(col + 1) } else { None };
        
        let mut neighbours: Vec<Coord> = Vec::with_capacity(8);
        let mut mine_count: usize = 0;
        
        for adjacent_coord in [
            (above,     left), (above, Some(col)), (above,     right),
            (Some(row), left),                     (Some(row), right),
            (below,     left), (below, Some(col)), (below,     right) ]  {
            if let (Some(neighbour_row), Some(neighbour_col)) = adjacent_coord {
                if !self.explored.contains(&(neighbour_row, neighbour_col))
                    && !self.unexplored.contains(&(neighbour_row, neighbour_col)) {
                        neighbours.push((neighbour_row, neighbour_col));
                        mine_count += self.mines_at.contains(&(neighbour_row, neighbour_col)) as usize;
                    }
            }
        }

        if mine_count > 0 {
            self.update_label((row, col), mine_count.to_string().chars().next().unwrap());
            return ExploreResult::MinedNeighbourhood;
        }
        
        // 3. Neighbourhood is not mined:
        // Add unexplored neighbours to the cache.
        self.update_label((row, col), ' ');
        self.unexplored.extend(neighbours);
        
        ExploreResult::Clear
    }
}


enum ParseResult<'a> {
    Ok,
    TooManyArguments,
    MissingArgument,
    NotNatural(&'a str)
}

fn parse_arguments<'a>(line: &'a str, args: &mut [usize], mandatory: bool) -> ParseResult<'a> {

    let mut args_it = args.iter_mut();

    for slice in line.split(',') {

        let arg = args_it.next();
        
        if arg.is_none() {
            return ParseResult::TooManyArguments;
        }
        
        match slice.parse::<usize>() {
            Ok(n) => *arg.unwrap() = n,

            // If `mandatory' is false, do not consider missing arguments as errors.
            Err(error)
                if matches!(error.kind(), std::num::IntErrorKind::Empty) => {
                    if mandatory {
                        return ParseResult::MissingArgument;
                    }
                },
            
            _ => {
                return ParseResult::NotNatural(slice);
            }
        }
    }

    ParseResult::Ok

}    


fn main() {

    /* List of available commands
     * n   rows, cols, %  start a new game
     * x   row,  col      explore the cell at (row, col)
     * >/f row,  col      flag the cell at (row, col)
     * h                  print the list of commands
     * q                  quit the game
     */

    println!("Welcome to rmines.");
    let prefix: &'static str = ">>";

    let mut board = Board::new(10, 10, 50).unwrap();
    let mut line = String::new();
    let mut rng = rand::thread_rng();    

    'main:
    loop {

        // Print the board and other game-related information.
        print!("{}\n>: {flagged} of {mine_count} mines\n\n{prefix} ",
               board, flagged = board.get_flagged_count(),
               mine_count = board.get_mine_count());
        io::stdout().flush().unwrap();
        
        line.clear();
        match io::stdin().read_line(&mut line) {

            Ok(_) => {

                line.retain(|c| !c.is_whitespace());

                if let Some(cmd) = line.chars().next() {

                    // Arguments are mandatory for 'f/>' but completely optional for 
                    // the 'n' and 'x' commands. In this latter case, any missing argument
                    // is replaced by a random value chosen appropriately. All arguments
                    // must be convertible to `usize'.

                    let arg_line = line.strip_prefix(cmd).unwrap();

                    match cmd {

                        'n' => {

                            let mut args: [usize; 3] = [
                                rng.gen_range(1..=board.get_rows()), // row count
                                rng.gen_range(1..=board.get_cols()), // col count
                                0
                            ];

                            // Default number of mines.
                            args[2] = rng.gen_range(1..(args[0] * args[1]));

                            match parse_arguments(arg_line, &mut args, false) {
                                ParseResult::TooManyArguments => {
                                    println!("{prefix} '{cmd}': too many arguments, expected\
                                              at most three: `[rows]', `[columns]', and `[mine count]'.");
                                    continue;
                                }

                                ParseResult::NotNatural(slice) => {
                                    println!("{prefix} '{cmd}': expected a positive number,\
                                              but got '{slice}' instead");
                                    continue;
                                },

                                _ => {}
                            }

                            // (Attempt to) create a new board.
                            match Board::new(args[0], args[1], args[2]) {
                                Ok(new_board) => {
                                    println!("{prefix} Starting a new game. The new board has \
                                              {rows} rows, {cols} columns, and ~{count} mines.",
                                             rows = args[0], cols = args[1], count = args[2]);
                                    board = new_board;
                                },

                                Err(BoardError::NullArea) => {
                                    println!("{prefix} '{cmd}': Cannot create a board with zero rows or columns!");
                                },

                                Err(BoardError::TooManyMines) => {
                                    println!("{prefix} '{cmd}': Too many mines for such small board!");
                                }
                            }
                        },

                        'x' => { // Explore the cell at the given coordinate.
                            
                            let mut args: [usize; 2] = [
                                rng.gen_range(1..=board.get_rows()), // row
                                rng.gen_range(1..=board.get_cols()), // col
                            ];

                            match parse_arguments(arg_line, &mut args, false) {
                                ParseResult::TooManyArguments => {
                                    println!("{prefix} '{cmd}': too many arguments, expected \
                                              at most two: `[row]', `[colum]'.");
                                    continue;
                                },

                                ParseResult::NotNatural(slice) => {
                                    println!("{prefix} '{cmd}': expected a positive number, \
                                              but got '{slice}' instead");
                                    continue;
                                },

                                _ => {}
                            }
                            
                            // (Attempt to) add the specified coordinate to the unexplored cache.
                            if !board.add_to_unexplored((args[0] - 1, args[1] - 1)) {
                                println!("{prefix} '{cmd}': coordinate ({x}, {y}) out of bounds.",
                                         x = args[0], y = args[1]);
                                continue 'main;
                            }
                            
                            // board.explore() explores the board greedily, i.e., it keeps exploring
                            // empty neighbourhoods until it runs into one that is mined.

                            loop {
                                match board.explore() {
                                    ExploreResult::EmptyCache => break,
                                    ExploreResult::Mined => {
                                        println!("{prefix} Oh no! You found a mine! Here's \
                                                  the board:\n\n{board}\nGame over!");
                                        // TODO: ask the user if they want to start a new game.
                                        break 'main;
                                    },
                                    _ => {}
                                }
                            }
                                
                            // The user wins when the number of unexplored cells equals the
                            // number of mines.

                            if board.all_mines_located() {
                                println!("{prefix} Congratulations! All mines have been located!");
                                break 'main;
                            }
                        },

                        'f' | '>' => { // Flag the cell at the coordinate given.

                            let mut args: [usize; 2] = [ 0; 2 ];

                            match parse_arguments(arg_line, &mut args, true) {
                                ParseResult::MissingArgument => {
                                    println!("{prefix} '{cmd}': too few arguments passed in.");
                                    continue;
                                },

                                ParseResult::TooManyArguments => {
                                    println!("{prefix} '{cmd}': too many arguments, expected \
                                              at most two: `[row]', `[colum]'.");
                                    continue;
                                },

                                ParseResult::NotNatural(slice) => {
                                    println!("{prefix} '{cmd}': expected a positive number, \
                                              but got '{slice}' instead");
                                    continue;
                                },

                                _ => {}
                            }

                            if !board.toggle_flag_at((args[0] - 1, args[1] - 1)) {
                                println!("{prefix} '{cmd}': the chosen cell is off the board!");
                                continue 'main;
                            }
                        },

                        'h' | '?' =>  { // Print the list of available commands.

                            if !arg_line.is_empty() {
                                println!("{prefix} '{cmd}': unknown command. Did you mean 'h'?");
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
                                println!("{prefix} '{cmd}': unknown command. Did you mean 'q'?");
                            }

                            println!("Goodbye!");
                            break;
                        },

                        _ => { // Unknown command passed in.
                            println!("{prefix} Unknown commmand '{cmd}'.");
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

#[cfg(test)]
mod tests {
    use super::*;
    use rand::prelude::*;

    #[test]
    fn test_explore() {

        // Create a new Board and mine it.
        let mut rng = rand::thread_rng();
        let rows: usize = rng.gen_range(1..=10);
        let cols: usize = rng.gen_range(1..=10);
        let mine_count: usize = rng.gen_range(1..(rows * cols));

        let board_result = Board::new(rows, cols, mine_count);
        let mut board = board_result
            .unwrap_or_else(|_| panic!("Error while creating a \
                                        new Board ({rows}, {cols}, \
                                        {mine_count}).")
        );

        println!("Created board with ({rows}, {cols}, {mine_count}).");
        
        // Pick a coordinate at random.
        let index = rng.gen_range(0..(rows * cols));
        let coord = (index/board.cols, index % board.cols);

        println!("Exploring cell at ({}, {})", coord.0 + 1, coord.1 + 1);
        board.add_to_unexplored(coord);

        loop {
            match board.explore() {
                ExploreResult::Clear => { println!("Clear"); },
                ExploreResult::Mined => { println!("Mine found at ({}, {})", coord.0, coord.1); }
                ExploreResult::EmptyCache => { println!("Empty cache."); break; }
                ExploreResult::MinedNeighbourhood => { println!("MinedNeighbourhood"); }
            }
        }

        println!("{}", board);

    }
}
