// 

use std::{collections::{HashMap, HashSet}, fmt};
use rand::distributions::{Distribution, Uniform};

pub type Coord = (usize, usize);

#[derive(Debug)]
pub enum BoardError {
    NullArea,
    TooManyMines,
}

pub enum ExploreResult {
    Ok,
    EmptyCache,
    Mined,
    MinedNeighbourhood,
    Clear,
}

pub struct Board {
    // Dimensions of the board.
    rows: usize,
    cols: usize,
    area: usize,

    // Number of cells marked as mined.
    flagged: usize,

    // Cells to be explored in the next call to Board::explore().
    cached: HashSet<Coord>,

    // Cells already explored (and clear).
    clear: HashSet<Coord>,

    // Location of each coordinate on the board.
    mines_at: HashSet<Coord>,

    // The string representation of the board.
    board_string: String,

    // The label of each coordinate in the `board_string' array.
    labels: HashMap<Coord, usize>,
}

impl fmt::Display for Board {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.board_string)
    }
}

impl Board {

    pub fn new(rows: usize, cols: usize, mine_count: usize) -> Result<Self, BoardError> {

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
            cached: HashSet::with_capacity(board_area - mines_at.len()),
            clear: HashSet::with_capacity(board_area - mines_at.len()),
            mines_at,
            labels: board_string
                .match_indices('.')
                .enumerate()
                .map(|(n, (index, _))| ((n/cols, n % cols), index))
                .collect::<HashMap<Coord, usize>>(),
            board_string,
        })
    }

    pub fn cache(&mut self, coord: Coord) -> bool {

        if coord.0 < self.rows + 1 && coord.1 < self.cols + 1 {
            self.cached.insert((coord.0 - 1, coord.1 - 1));
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

    pub fn get_rows(&self) -> usize {
        self.rows
    }

    pub fn get_cols(&self) -> usize {
        self.cols
    }

    pub fn get_mine_count(&self) -> usize {
        self.mines_at.len()
    }

    pub fn get_flagged_count(&self) -> usize {
        self.flagged
    }

    pub fn all_mines_found(&self) -> bool {
        self.area - self.clear.len() == self.mines_at.len()
    }
    
    pub fn toggle_flag_at(&mut self, at: Coord) -> bool {

        if let Some(index) = self.labels.get(&at) {
            
            // Do nothing if the parcel has been explored.
            if !self.clear.contains(&at) {
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

    pub fn explore(&mut self) -> ExploreResult {

        // Get the next cell to explore.
        let Some(&(row, col)) = self.cached.iter().next()
        else { return ExploreResult::EmptyCache };
        self.cached.remove(&(row, col));

        // If false, the cell has been explored -- nothing to do.
        if !self.clear.insert((row, col)) {
            return ExploreResult::Clear;
        }

        // The cell is mined.
        if self.mines_at.contains(&(row, col)) {
            self.reveal_mines();
            return ExploreResult::Mined;
        }

        // Possible coordinates of each neighbor.
        let above = row.checked_sub(1);
        let below = if row + 1 < self.rows { Some(row + 1) } else { None };
        let left = col.checked_sub(1);
        let right = if col + 1 < self.cols { Some(col + 1) } else { None };

        // Neighbors not yet explored and candidate for exploration.
        let mut unexplored: Vec<Coord> = Vec::with_capacity(8);

        // Number of mines found in the neighborhood.
        let mut mined: usize = 0;

        // Immutable variable defined to improve readability.
        let neighborhood =
            [ (above,      left), (above, Some(col)), (above,     right),
               (Some(row), left),                     (Some(row), right),
               (below,     left), (below, Some(col)), (below,     right) ];

        for neighbor in neighborhood {

            // Filter neighbors with valid coordinates.
            if let (Some(ng_row), Some(ng_col)) = neighbor {

                if self.mines_at.contains(&(ng_row, ng_col)) {
                    mined += 1; // Mined neighbor.
                }
                else if mined == 0 && !self.clear.contains(&(ng_row, ng_col)) {
                    // If no mines have been found in the neighborhood yet, and the current cell has
                    // not been explored, then make it a candidate for exploration in a subsequent
                    // call to this function.
                    unexplored.push((ng_row, ng_col));
                }
            }
        }

        if mined > 0 {
            self.update_label((row, col), mined.to_string().chars().next().unwrap());
            return ExploreResult::MinedNeighbourhood;
        }
        
        // 3. Neighbourhood is not mined:
        // Add unexplored neighbours to the cache.
        self.update_label((row, col), ' ');
        self.cached.extend(unexplored);
        
        ExploreResult::Ok
    }
}
