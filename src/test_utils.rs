use super::*;

/// Creates a test function that runs the tests for all known types of
/// mazes.
#[macro_export]
macro_rules! maze_test {
    ($name:ident, $code:item) => {
        #[test]
        fn $name() {
            let width = 10;
            let height = 5;

            $code

            test(&mut shape::hex::Maze::new(width, height));
            test(&mut shape::quad::Maze::new(width, height));
            test(&mut shape::tri::Maze::new(width, height));
        }
    }
}

/// Determines whether two physical locations are close enough to be
/// considered equal.
///
/// # Arguments
/// * `expected` - The expected location.
/// * `actual` - Another location.
pub fn is_close(expected: physical::Pos, actual: physical::Pos) -> bool {
    let d = (expected.0 - actual.0, expected.1 - actual.1);
    (d.0 * d.0 + d.1 * d.1).sqrt() < 0.00001
}

/// A navigator through a maze.
///
/// This struct provides utility methods to open and close doors based on
/// directions.
pub struct Navigator<'a> {
    maze: &'a mut Maze,
    pos: Option<matrix::Pos>,
    log: Vec<matrix::Pos>,
}

impl<'a> Navigator<'a> {
    /// Creates a new navigator for a specific maze.
    ///
    /// # Arguments
    /// *  `maze` - The maze to modify.
    pub fn new(maze: &'a mut Maze) -> Navigator<'a> {
        Navigator {
            maze: maze,
            pos: None,
            log: Vec::new(),
        }
    }

    /// Moves the navigator to a specific room.
    ///
    /// # Arguments
    /// *  `pos` - The new position.
    pub fn from(mut self, pos: matrix::Pos) -> Self {
        self.pos = Some(pos);
        self
    }

    /// Opens or closes a wall leading _up_.
    ///
    /// The current room position is also updated.
    ///
    /// # Arguments
    /// *  `open` - Whether to open the wall.
    ///
    /// # Panics
    /// This method panics if there is no wall leading up from the current
    /// room.
    pub fn up(self, open: bool) -> Self {
        self.navigate(|wall| wall.dir == (0, -1), open)
    }

    /// Opens or closes a wall leading _down_.
    ///
    /// The current room position is also updated.
    ///
    /// # Arguments
    /// *  `open` - Whether to open the wall.
    ///
    /// # Panics
    /// This method panics if there is no wall leading down from the current
    /// room.
    pub fn down(self, open: bool) -> Self {
        self.navigate(|wall| wall.dir == (0, 1), open)
    }

    /// Opens or closes a wall leading _left_.
    ///
    /// The current room position is also updated.
    ///
    /// # Arguments
    /// *  `open` - Whether to open the wall.
    ///
    /// # Panics
    /// This method panics if there is no wall leading left from the current
    /// room.
    pub fn left(self, open: bool) -> Self {
        self.navigate(|wall| wall.dir == (-1, 0), open)
    }

    /// Opens or closes a wall leading _right_.
    ///
    /// The current room position is also updated.
    ///
    /// # Arguments
    /// *  `open` - Whether to open the wall.
    ///
    /// # Panics
    /// This method panics if there is no wall leading right from the
    /// current room.
    pub fn right(self, open: bool) -> Self {
        self.navigate(|wall| wall.dir == (1, 0), open)
    }

    /// Stops and freezes this navigator.
    pub fn stop(mut self) -> Vec<matrix::Pos> {
        self.log.push(self.pos.unwrap());
        self.log
    }

    /// Opens or closes a wall.
    ///
    /// The current room position is also updated.
    ///
    /// # Arguments
    /// *  `open` - Whether to open the wall.
    ///
    /// The wall selected is the first one for which `predicate` returns
    /// `true`.
    ///
    /// # Panics
    /// This method panics if there is no wall for which the predicate
    /// returns `true`.
    pub fn navigate<P>(mut self, mut predicate: P, open: bool) -> Self
    where
        for<'r> P: FnMut(&'r &&wall::Wall) -> bool,
    {
        if self.pos.is_none() {
            self.pos = self.maze
                .rooms()
                .positions()
                .filter(|&pos| {
                    self.maze
                        .walls(pos)
                        .iter()
                        .any(|wall| predicate(&wall))
                })
                .next();
        }
        let pos = self.pos.unwrap();
        self.log.push(pos);

        let wall = self.maze
            .walls(pos)
            .iter()
            .filter(predicate)
            .filter(|wall| {
                self.maze
                    .rooms()
                    .is_inside((pos.0 + wall.dir.0, pos.1 + wall.dir.1))
            })
            .next()
            .unwrap();
        self.maze.set_open((pos, wall), open);
        self.pos = Some((pos.0 + wall.dir.0, pos.1 + wall.dir.1));
        self
    }
}
