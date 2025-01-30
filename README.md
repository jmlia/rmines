# rmines v0.1.0

`rmines` is an implementation in pure Rust of the classic game _Minesweeper_. As
a learner of Rust, I thought that programming the logic of this game would be an
interesting challenge to take on.

## Dependencies

This program requires only the `rand` crate.

## How to play

Just run `cargo run` to start a new game with a default board of 10x10 cells and
(approximately) 50 mines placed at random. The following commands are supported:

- `n <rows>, <cols>, <mine_count>`: creates a new board with dimensions
  `<rows>x<cols>` and (approximately) `<mine_count>` mines.
- `x <row>, <col>`: explore the cell at `(<row>, <col>)`.
- `f/> <row>, <col>`: flag the cell at `(<row>, <col>)` as mined.
- `h/?`: print the list of available commands.
- `q`: quit the game.
  
Arguments to the `n`and `x` commands are optional. If not given, appropriate
values for them will be chosen at random.

## TODO

- Keep track of and print the total playing time.
- Ask the user if they would like to start a new game after the current one is over.
- Give the user the possibility of saving their progress and resume the game
  later.
- Move the `Board` structure and its `impl` block to a separate crate.
- Write many more tests!
- Refactor the `explore` function to make it more efficient.
- Make the user interface more functional (perhaps through third-patry crates
  like `rustyline`).
- Make sure that every new board is exactly populated with `<mine_count>` mines
   (see the comments in `Board::new()` for details).
