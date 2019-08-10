use crate::Maze;

use crate::matrix;

/// Initialises a maze by clearing all inner walls.
///
/// This method will ignore rooms for which `filter` returns `false`.
///
/// # Arguments
/// *  `_rng` - Not used.
/// *  `filter` - A predicate filtering rooms to consider.
pub fn initialize<F, R>(mut maze: Maze, _rng: &mut R, filter: F) -> Maze
where
    F: Fn(matrix::Pos) -> bool,
    R: super::Randomizer + Sized,
{
    let (count, candidates) =
        matrix::filter(maze.width(), maze.height(), filter);
    if count == 0 {
        return maze;
    }

    for pos in maze.rooms().positions().filter(|&pos| candidates[pos]) {
        for wall in maze.walls(pos) {
            let (pos, wall) = maze.back((pos, wall));
            if *candidates.get(pos).unwrap_or(&false) {
                maze.open((pos, wall));
            }
        }
    }

    maze
}
