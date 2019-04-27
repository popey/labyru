use std::f32::consts::PI;

use crate::WallPos;

use crate::matrix;
use crate::physical;
use crate::wall;

/// A span step angle
const D: f32 = PI / 6.0;

/// D.cos()
const D_COS: f32 = 0.866_025_4f32;

/// D.sin()
const D_SIN: f32 = 1.0 / 2.0;

/// The distance between the centre of a room and the centre of a room on the
/// next row.
const HORIZONTAL_MULTIPLICATOR: f32 = 2.0 * D_COS;

/// The distance between the centre of a room and the centre of a room on the
/// next column.
const VERTICAL_MULTIPLICATOR: f32 = 2.0 - D_SIN;

const TOP_HEIGHT: f32 = 1.0 + D_SIN;
const GRADIENT: f32 = (1.0 + D_SIN) / D_COS;

// The walls are arranged in back-to-back pairs
define_shape! {
    LEFT0 = {
        corner_wall_offsets: &[
            ((-1, 0), WallIndex::DOWN_RIGHT0 as usize),
            ((0, 1), WallIndex::UP_RIGHT1 as usize),
        ],
        dir: (-1, 0),
        span: (5.0 * D, 7.0 * D),
    },
    RIGHT0 = {
        corner_wall_offsets: &[
            ((1, 0), WallIndex::UP_LEFT0 as usize),
            ((1, -1), WallIndex::DOWN_LEFT1 as usize),
        ],
        dir: (1, 0),
        span: (11.0 * D, 13.0 * D),
    },

    LEFT1 = {
        corner_wall_offsets: &[
            ((-1, 0), WallIndex::DOWN_RIGHT1 as usize),
            ((-1, 1), WallIndex::UP_RIGHT0 as usize),
        ],
        dir: (-1, 0),
        span: (5.0 * D, 7.0 * D),
    },
    RIGHT1 = {
        corner_wall_offsets: &[
            ((1, 0), WallIndex::UP_LEFT1 as usize),
            ((0, -1), WallIndex::DOWN_LEFT0 as usize),
        ],
        dir: (1, 0),
        span: (11.0 * D, 13.0 * D),
    },

    UP_LEFT0 = {
        corner_wall_offsets: &[
            ((0, -1), WallIndex::DOWN_LEFT1 as usize),
            ((-1, 0), WallIndex::UP_RIGHT0 as usize),
        ],
        dir: (0, -1),
        span: (7.0 * D, 9.0 * D),
    },
    DOWN_RIGHT1 = {
        corner_wall_offsets: &[
            ((0, 1), WallIndex::UP_RIGHT0 as usize),
            ((1, 0), WallIndex::LEFT1 as usize),
        ],
        dir: (0, 1),
        span: (1.0 * D, 3.0 * D),
    },

    UP_LEFT1 = {
        corner_wall_offsets: &[
            ((-1, -1), WallIndex::DOWN_LEFT0 as usize),
            ((-1, 0), WallIndex::RIGHT1 as usize),
        ],
        dir: (-1, -1),
        span: (7.0 * D, 9.0 * D),
    },
    DOWN_RIGHT0 = {
        corner_wall_offsets: &[
            ((1, 1), WallIndex::UP_RIGHT1 as usize),
            ((1, 0), WallIndex::LEFT0 as usize),
        ],
        dir: (1, 1),
        span: (1.0 * D, 3.0 * D),
    },

    UP_RIGHT0 = {
        corner_wall_offsets: &[
            ((1, -1), WallIndex::LEFT1 as usize),
            ((0, -1), WallIndex::DOWN_RIGHT1 as usize),
        ],
        dir: (1, -1),
        span: (9.0 * D, 11.0 * D),
    },
    DOWN_LEFT1 = {
        corner_wall_offsets: &[
            ((-1, 1), WallIndex::RIGHT0 as usize),
            ((0, 1), WallIndex::UP_LEFT0 as usize),
        ],
        dir: (-1, 1),
        span: (3.0 * D, 5.0 * D),
    },

    UP_RIGHT1 = {
        corner_wall_offsets: &[
            ((0, -1), WallIndex::LEFT0 as usize),
            ((-1, -1), WallIndex::DOWN_RIGHT0 as usize),
        ],
        dir: (0, -1),
        span: (9.0 * D, 11.0 * D),
    },
    DOWN_LEFT0 = {
        corner_wall_offsets: &[
            ((0, 1), WallIndex::RIGHT1 as usize),
            ((1, 1), WallIndex::UP_LEFT1 as usize),
        ],
        dir: (0, 1),
        span: (3.0 * D, 5.0 * D),
    }
}

/// The walls for even rows
static ALL0: &[&wall::Wall] = &[
    &walls::LEFT0,
    &walls::UP_LEFT0,
    &walls::UP_RIGHT0,
    &walls::RIGHT0,
    &walls::DOWN_RIGHT0,
    &walls::DOWN_LEFT0,
];

/// The walls for odd rows
static ALL1: &[&wall::Wall] = &[
    &walls::LEFT1,
    &walls::UP_LEFT1,
    &walls::UP_RIGHT1,
    &walls::RIGHT1,
    &walls::DOWN_RIGHT1,
    &walls::DOWN_LEFT1,
];

pub fn back_index(wall: usize) -> usize {
    wall ^ 0b0001
}

pub fn opposite(wall_pos: WallPos) -> Option<&'static wall::Wall> {
    let (_, wall) = wall_pos;

    // The left and right walls are back-to-back
    Some(
        walls::ALL[if (wall.index & !0b0011) == 0 {
            wall.index ^ 0b0001
        } else {
            wall.index ^ 0b0011
        }],
    )
}

pub fn walls(pos: matrix::Pos) -> &'static [&'static wall::Wall] {
    if pos.row & 1 == 1 {
        &ALL1
    } else {
        &ALL0
    }
}

pub fn center(pos: matrix::Pos) -> physical::Pos {
    physical::Pos {
        x: (pos.col as f32 + if pos.row & 1 == 1 { 0.5 } else { 1.0 })
            * HORIZONTAL_MULTIPLICATOR,
        y: (pos.row as f32) * VERTICAL_MULTIPLICATOR + 1.0,
    }
}

pub fn room_at(pos: physical::Pos) -> matrix::Pos {
    // Calculate approximations of the room position
    let approx_row = (pos.y / VERTICAL_MULTIPLICATOR).floor();
    let row_odd = approx_row as i32 & 1 == 1;
    let approx_col = if row_odd {
        (pos.x / HORIZONTAL_MULTIPLICATOR)
    } else {
        (pos.x / HORIZONTAL_MULTIPLICATOR - 0.5)
    };

    // Calculate relative positions within the room
    let rel_y = pos.y - (approx_row * VERTICAL_MULTIPLICATOR);
    let rel_x = if row_odd {
        (pos.x - ((approx_col - 0.5) * HORIZONTAL_MULTIPLICATOR))
    } else {
        (pos.x - (approx_col * HORIZONTAL_MULTIPLICATOR))
    };

    if rel_y < (-GRADIENT * rel_x) + TOP_HEIGHT {
        matrix::Pos {
            col: approx_col as isize - !row_odd as isize,
            row: approx_row as isize - 1,
        }
    } else if rel_y < (GRADIENT * rel_x) - TOP_HEIGHT {
        matrix::Pos {
            col: approx_col as isize + row_odd as isize,
            row: approx_row as isize - 1,
        }
    } else {
        matrix::Pos {
            col: approx_col as isize,
            row: approx_row as isize,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::walls;
    use crate::test_utils::*;
    use crate::Shape;
    use crate::WallPos;

    #[test]
    fn back() {
        let maze = maze(5, 5);

        assert_eq!(
            maze.back((matrix_pos(1, 0), &walls::LEFT0)),
            (matrix_pos(0, 0), &walls::RIGHT0)
        );
        assert_eq!(
            maze.back((matrix_pos(1, 1), &walls::LEFT1)),
            (matrix_pos(0, 1), &walls::RIGHT1)
        );
        assert_eq!(
            maze.back((matrix_pos(1, 2), &walls::UP_LEFT0)),
            (matrix_pos(1, 1), &walls::DOWN_RIGHT1,)
        );
        assert_eq!(
            maze.back((matrix_pos(1, 1), &walls::UP_LEFT1)),
            (matrix_pos(0, 0), &walls::DOWN_RIGHT0,)
        );
        assert_eq!(
            maze.back((matrix_pos(0, 2), &walls::UP_RIGHT0)),
            (matrix_pos(1, 1), &walls::DOWN_LEFT1,)
        );
        assert_eq!(
            maze.back((matrix_pos(0, 1), &walls::UP_RIGHT1)),
            (matrix_pos(0, 0), &walls::DOWN_LEFT0,)
        );
        assert_eq!(
            maze.back((matrix_pos(0, 0), &walls::RIGHT0)),
            (matrix_pos(1, 0), &walls::LEFT0)
        );
        assert_eq!(
            maze.back((matrix_pos(0, 1), &walls::RIGHT1)),
            (matrix_pos(1, 1), &walls::LEFT1)
        );
        assert_eq!(
            maze.back((matrix_pos(0, 0), &walls::DOWN_RIGHT0)),
            (matrix_pos(1, 1), &walls::UP_LEFT1,)
        );
        assert_eq!(
            maze.back((matrix_pos(0, 1), &walls::DOWN_RIGHT1)),
            (matrix_pos(0, 2), &walls::UP_LEFT0,)
        );
        assert_eq!(
            maze.back((matrix_pos(1, 0), &walls::DOWN_LEFT0)),
            (matrix_pos(1, 1), &walls::UP_RIGHT1,)
        );
        assert_eq!(
            maze.back((matrix_pos(1, 1), &walls::DOWN_LEFT1)),
            (matrix_pos(0, 2), &walls::UP_RIGHT0,)
        );
    }

    #[test]
    fn opposite() {
        let maze = maze(5, 5);

        assert_eq!(
            maze.opposite((matrix_pos(1, 0), &walls::LEFT0)).unwrap(),
            &walls::RIGHT0
        );
        assert_eq!(
            maze.opposite((matrix_pos(1, 1), &walls::LEFT1)).unwrap(),
            &walls::RIGHT1
        );
        assert_eq!(
            maze.opposite((matrix_pos(1, 2), &walls::UP_LEFT0)).unwrap(),
            &walls::DOWN_RIGHT0
        );
        assert_eq!(
            maze.opposite((matrix_pos(1, 1), &walls::UP_LEFT1)).unwrap(),
            &walls::DOWN_RIGHT1
        );
        assert_eq!(
            maze.opposite((matrix_pos(0, 2), &walls::UP_RIGHT0))
                .unwrap(),
            &walls::DOWN_LEFT0
        );
        assert_eq!(
            maze.opposite((matrix_pos(0, 1), &walls::UP_RIGHT1))
                .unwrap(),
            &walls::DOWN_LEFT1
        );
        assert_eq!(
            maze.opposite((matrix_pos(0, 0), &walls::RIGHT0)).unwrap(),
            &walls::LEFT0
        );
        assert_eq!(
            maze.opposite((matrix_pos(0, 1), &walls::RIGHT1)).unwrap(),
            &walls::LEFT1
        );
        assert_eq!(
            maze.opposite((matrix_pos(0, 0), &walls::DOWN_RIGHT0))
                .unwrap(),
            &walls::UP_LEFT0
        );
        assert_eq!(
            maze.opposite((matrix_pos(0, 1), &walls::DOWN_RIGHT1))
                .unwrap(),
            &walls::UP_LEFT1
        );
        assert_eq!(
            maze.opposite((matrix_pos(1, 0), &walls::DOWN_LEFT0))
                .unwrap(),
            &walls::UP_RIGHT0
        );
        assert_eq!(
            maze.opposite((matrix_pos(1, 1), &walls::DOWN_LEFT1))
                .unwrap(),
            &walls::UP_RIGHT1
        );
    }

    #[test]
    fn corner_walls() {
        let maze = maze(5, 5);

        assert_eq!(
            maze.corner_walls((matrix_pos(1, 2), &walls::LEFT0)),
            vec![
                (matrix_pos(1, 2), &walls::LEFT0),
                (matrix_pos(0, 2), &walls::DOWN_RIGHT0),
                (matrix_pos(1, 3), &walls::UP_RIGHT1),
            ]
        );
        assert_eq!(
            maze.corner_walls((matrix_pos(1, 1), &walls::LEFT1)),
            vec![
                (matrix_pos(1, 1), &walls::LEFT1),
                (matrix_pos(0, 1), &walls::DOWN_RIGHT1),
                (matrix_pos(0, 2), &walls::UP_RIGHT0),
            ]
        );
        assert_eq!(
            maze.corner_walls((matrix_pos(1, 2), &walls::UP_LEFT0)),
            vec![
                (matrix_pos(1, 2), &walls::UP_LEFT0),
                (matrix_pos(1, 1), &walls::DOWN_LEFT1),
                (matrix_pos(0, 2), &walls::UP_RIGHT0),
            ]
        );
        assert_eq!(
            maze.corner_walls((matrix_pos(1, 1), &walls::UP_LEFT1)),
            vec![
                (matrix_pos(1, 1), &walls::UP_LEFT1),
                (matrix_pos(0, 0), &walls::DOWN_LEFT0),
                (matrix_pos(0, 1), &walls::RIGHT1),
            ]
        );
        assert_eq!(
            maze.corner_walls((matrix_pos(1, 2), &walls::UP_RIGHT0)),
            vec![
                (matrix_pos(1, 2), &walls::UP_RIGHT0),
                (matrix_pos(2, 1), &walls::LEFT1),
                (matrix_pos(1, 1), &walls::DOWN_RIGHT1),
            ]
        );
        assert_eq!(
            maze.corner_walls((matrix_pos(1, 1), &walls::UP_RIGHT1)),
            vec![
                (matrix_pos(1, 1), &walls::UP_RIGHT1),
                (matrix_pos(1, 0), &walls::LEFT0),
                (matrix_pos(0, 0), &walls::DOWN_RIGHT0),
            ]
        );
        assert_eq!(
            maze.corner_walls((matrix_pos(1, 2), &walls::RIGHT0)),
            vec![
                (matrix_pos(1, 2), &walls::RIGHT0),
                (matrix_pos(2, 2), &walls::UP_LEFT0),
                (matrix_pos(2, 1), &walls::DOWN_LEFT1),
            ]
        );
        assert_eq!(
            maze.corner_walls((matrix_pos(1, 1), &walls::RIGHT1)),
            vec![
                (matrix_pos(1, 1), &walls::RIGHT1),
                (matrix_pos(2, 1), &walls::UP_LEFT1),
                (matrix_pos(1, 0), &walls::DOWN_LEFT0),
            ]
        );
        assert_eq!(
            maze.corner_walls((matrix_pos(1, 2), &walls::DOWN_RIGHT0)),
            vec![
                (matrix_pos(1, 2), &walls::DOWN_RIGHT0),
                (matrix_pos(2, 3), &walls::UP_RIGHT1),
                (matrix_pos(2, 2), &walls::LEFT0),
            ]
        );
        assert_eq!(
            maze.corner_walls((matrix_pos(1, 1), &walls::DOWN_RIGHT1)),
            vec![
                (matrix_pos(1, 1), &walls::DOWN_RIGHT1),
                (matrix_pos(1, 2), &walls::UP_RIGHT0),
                (matrix_pos(2, 1), &walls::LEFT1),
            ]
        );
        assert_eq!(
            maze.corner_walls((matrix_pos(1, 2), &walls::DOWN_LEFT0)),
            vec![
                (matrix_pos(1, 2), &walls::DOWN_LEFT0),
                (matrix_pos(1, 3), &walls::RIGHT1),
                (matrix_pos(2, 3), &walls::UP_LEFT1),
            ]
        );
        assert_eq!(
            maze.corner_walls((matrix_pos(1, 1), &walls::DOWN_LEFT1)),
            vec![
                (matrix_pos(1, 1), &walls::DOWN_LEFT1),
                (matrix_pos(0, 2), &walls::RIGHT0),
                (matrix_pos(1, 2), &walls::UP_LEFT0),
            ]
        );
    }

    #[test]
    fn follow_wall_single_room() {
        let maze = maze(5, 5);

        assert_eq!(
            vec![
                (matrix_pos(0, 0), &walls::LEFT0),
                (matrix_pos(0, 0), &walls::UP_LEFT0),
                (matrix_pos(0, 0), &walls::UP_RIGHT0),
                (matrix_pos(0, 0), &walls::RIGHT0),
                (matrix_pos(0, 0), &walls::DOWN_RIGHT0),
                (matrix_pos(0, 0), &walls::DOWN_LEFT0),
            ],
            maze.follow_wall((matrix_pos(0, 0), &walls::LEFT0))
                .map(|(from, _)| from)
                .collect::<Vec<WallPos>>()
        );
    }

    #[test]
    fn follow_wall() {
        let mut maze = maze(5, 5);

        Navigator::new(&mut maze)
            .from(matrix_pos(0, 0))
            .down(true)
            .right(true)
            .up(true);

        assert_eq!(
            vec![
                (matrix_pos(0, 0), &walls::LEFT0),
                (matrix_pos(0, 0), &walls::UP_LEFT0),
                (matrix_pos(0, 0), &walls::UP_RIGHT0),
                (matrix_pos(0, 0), &walls::RIGHT0),
                (matrix_pos(0, 0), &walls::DOWN_RIGHT0),
                (matrix_pos(1, 1), &walls::UP_LEFT1),
                (matrix_pos(1, 0), &walls::LEFT0),
                (matrix_pos(1, 0), &walls::UP_LEFT0),
                (matrix_pos(1, 0), &walls::UP_RIGHT0),
                (matrix_pos(1, 0), &walls::RIGHT0),
                (matrix_pos(1, 0), &walls::DOWN_RIGHT0),
                (matrix_pos(1, 1), &walls::RIGHT1),
                (matrix_pos(1, 1), &walls::DOWN_RIGHT1),
                (matrix_pos(1, 1), &walls::DOWN_LEFT1),
                (matrix_pos(0, 1), &walls::DOWN_RIGHT1),
                (matrix_pos(0, 1), &walls::DOWN_LEFT1),
                (matrix_pos(0, 1), &walls::LEFT1),
                (matrix_pos(0, 1), &walls::UP_LEFT1),
            ],
            maze.follow_wall((matrix_pos(0, 0), &walls::LEFT0))
                .map(|(from, _)| from)
                .collect::<Vec<WallPos>>()
        );
    }

    /// Creates a maze.
    ///
    /// # Arguments
    /// *  `width` - The width.
    /// *  `height` - The height.
    fn maze(width: usize, height: usize) -> crate::Maze {
        crate::Maze::new(Shape::Hex, width, height)
    }
}
