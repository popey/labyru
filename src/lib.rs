extern crate rand;

#[cfg(feature = "render-svg")]
extern crate svg;

#[cfg(test)]
#[macro_use]
mod tests;

#[macro_use]
pub mod wall;

pub mod traits;
pub use traits::*;

pub mod matrix;
pub mod room;

mod util;


/// A wall of a room.
pub type WallPos = (matrix::Pos, &'static wall::Wall);


/// A maze contains rooms and has methods for managing paths and doors.
pub trait Maze: Physical + Renderable + shape::Shape + Walkable {
    /// Returns the width of the maze.
    ///
    /// This is short hand for `self.rooms().width()`.
    fn width(&self) -> usize {
        self.rooms().width
    }

    /// Returns the height of the maze.
    ///
    /// This is short hand for `self.rooms().height()`.
    fn height(&self) -> usize {
        self.rooms().height
    }

    /// Returns whether a specified wall is open.
    ///
    /// # Arguments
    /// * `wall_pos` - The wall position.
    fn is_open(&self, wall_pos: WallPos) -> bool {
        match self.rooms().get(wall_pos.0) {
            Some(room) => room.is_open(wall_pos.1),
            None => false,
        }
    }

    /// Returns whether two rooms are connected.
    ///
    /// Two rooms are connected if there is an open wall between them, or if
    /// they are the same room.
    ///
    /// # Arguments
    /// * `pos1` - The first room.
    /// * `pos2` - The second room.
    fn connected(&self, pos1: matrix::Pos, pos2: matrix::Pos) -> bool {
        if pos1 == pos2 {
            true
        } else if let Some(wall) = self.walls(pos1)
                   .iter()
                   .filter(
            |wall| (pos1.0 + wall.dir.0, pos1.1 + wall.dir.1) == pos2,
        )
                   .next()
        {
            self.is_open((pos1, wall))
        } else {
            false
        }
    }

    /// Sets whether a wall is open.
    ///
    /// # Arguments
    /// * `wall_pos` - The wall position.
    /// * `value` - Whether to open the wall.
    fn set_open(&mut self, wall_pos: WallPos, value: bool) {
        // First modify the requested wall...
        if let Some(room) = self.rooms_mut().get_mut(wall_pos.0) {
            room.set_open(wall_pos.1, value);
        }

        // ..and then sync the value on the back
        let other = self.back(wall_pos);
        if let Some(other_room) = self.rooms_mut().get_mut(other.0) {
            other_room.set_open(other.1, value);
        }
    }

    /// Opens a wall.
    ///
    /// # Arguments
    /// * `wall_pos` - The wall position.
    fn open(&mut self, wall_pos: WallPos) {
        self.set_open(wall_pos, true);
    }

    /// Closes a wall.
    ///
    /// # Arguments
    /// * `wall_pos` - The wall position.
    fn close(&mut self, wall_pos: WallPos) {
        self.set_open(wall_pos, false);
    }

    /// Retrieves a reference to the underlying rooms.
    fn rooms(&self) -> &room::Rooms;

    /// Retrieves a mutable reference to the underlying rooms.
    fn rooms_mut(&mut self) -> &mut room::Rooms;
}


pub mod initialize;
pub use initialize::*;

#[macro_use]
pub mod shape;
