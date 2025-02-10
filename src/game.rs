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

    pub fn add_to_unexplored(&mut self, coord: Coord) -> bool {

        if coord.0 * self.cols + coord.1 < self.area {
            self.unexplored.insert(coord);
            return true;
        }
        false
    }

    pub fn reveal_mines(&mut self) {
        for coord in &self.mines_at {
            let index = self.labels.get(coord).unwrap();
            self.board_string.replace_range(*index..(*index + 1), "*");
        }
    }

    pub fn update_label(&mut self, at: Coord, label: char) {
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

    pub fn all_mines_located(&self) -> bool {
        self.area - self.explored.len() == self.mines_at.len()
    }
    
    pub fn toggle_flag_at(&mut self, at: Coord) -> bool {

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

    pub fn explore(&mut self) -> ExploreResult {

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
