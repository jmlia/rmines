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
    BoardClear,
}

pub enum CacheResult {
    Ok,
    InvalidCoordinate,
    Clear,
}

pub enum CellLabel {
    Clear,
    Flag,
    MinedNeighbors(usize)
}

pub struct Board {
    // Dimensions of the board.
    rows: usize,
    cols: usize,
    area: usize,

    // Number of cells marked as mined.
    flagged: HashSet<Coord>,

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

        // Header including column labels and separators.
        board_string.push_str(format!("{:width$}|", 1,
                                      width = (row_label_width + 1) +
                                      col_label_width).as_str());

        for col in 2..(cols + 1) {
            board_string.push_str(format!("{:col_label_width$}|", col).as_str());
        }

        board_string.push('\n');

        // Row and cell labels.
        for row in 0..rows {
            board_string.push_str(format!("{:row_label_width$}|", row + 1).as_str());
            board_string.push_str(format!("{:>width$}", '.', width = col_label_width + 1)
                                  .repeat(cols).as_str());
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
            .collect();

        Ok(Board {
            rows,
            cols,
            area: board_area,
            flagged: HashSet::with_capacity(mines_at.len()),
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

    pub fn cache(&mut self, mut coord: Coord) -> CacheResult {

        // Coordinates as specified by the user are offset by 1.
        coord.0 -= 1;
        coord.1 -= 1;

        if !(coord.0 < self.rows && coord.1 < self.cols) {
            return CacheResult::InvalidCoordinate;
        }

        if self.clear.contains(&(coord.0, coord.1)) {
            return CacheResult::Clear;
        }

        self.cached.insert((coord.0, coord.1));
        CacheResult::Ok
    }

    fn reveal_mines(&mut self) {
        for coord in &self.mines_at {
            let &index = self.labels.get(coord).unwrap();
            self.board_string.replace_range(index..(index + 1), "*");
        }
    }

    pub fn update_label(&mut self, mut at: Coord, label: CellLabel, from_ui: bool) -> bool {

        // Coordinates passed in from UI calls are offset by (1, 1).
        if from_ui {
            at.0 -= 1;
            at.1 -= 1;
        }

        // Buffer to replace a single char in `board_string'.
        if let Some(&index) = self.labels.get(&at) {
   
            let mut buffer: [u8; 2] = [0; 2];
   
            match label {
                CellLabel::Clear => {
                    self.flagged.remove(&at);
                    self.board_string.replace_range(index..(index + 1),
                                                    ' '.encode_utf8(&mut buffer));
                },
                CellLabel::MinedNeighbors(mine_count) => {
                    self.flagged.remove(&at);
                    self.board_string.replace_range(index..(index + 1),
                                                    mine_count.to_string().chars()
                                                    .next().unwrap().encode_utf8(&mut buffer));
                },
                CellLabel::Flag => {
                    // Do nothing if the parcel has already been explored.
                    // Otherwise, exchange '>' for '.' and vice-versa.

                    if self.flagged.remove(&at) {
                        self.board_string.replace_range(index..(index + 1),
                                                        '.'.encode_utf8(&mut buffer));
                    }
                    else if self.flagged.len() < self.mines_at.len() {
                        self.flagged.insert(at);
                        self.board_string.replace_range(index..(index + 1),
                                                        '>'.encode_utf8(&mut buffer));
                    }
                }
            }

            return true;
        }

        false

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
        self.flagged.len()
    }

    pub fn explore(&mut self) -> ExploreResult {

        // Get the next cell to explore.
        // In passing, check if all mines have been found.

        if self.cached.is_empty() {

            if self.area - self.clear.len() == self.mines_at.len() {
                return ExploreResult::BoardClear;
            }
            else {
                return ExploreResult::EmptyCache;
            }
        }

        let &(row, col) = self.cached.iter().next().unwrap();
        self.cached.remove(&(row, col));

        // If the cell is mined, return.
        if self.mines_at.contains(&(row, col)) {
            self.reveal_mines();
            return ExploreResult::Mined;
        }

        self.clear.insert((row, col));

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
            self.update_label((row, col), CellLabel::MinedNeighbors(mined), false);
        }
        else {
            // Cache unexplored neighbors.
            self.update_label((row, col), CellLabel::Clear, false);
            self.cached.extend(unexplored);
        }

        ExploreResult::Ok
    }
}
