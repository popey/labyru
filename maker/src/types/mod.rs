use std;
use std::str;

use image;
use rayon;
use svg;

use rayon::prelude::*;
use svg::Node;

use maze;
use maze::initialize;
use maze::matrix;
use maze::matrix::AddableMatrix;
use maze_tools::image::Color;
use maze_tools::voronoi;

pub mod background_renderer;
pub use self::background_renderer::*;
pub mod break_post_processor;
pub use self::break_post_processor::*;
pub mod heatmap_renderer;
pub use self::heatmap_renderer::*;
pub mod mask_initializer;
pub use self::mask_initializer::*;
pub mod solve_renderer;
pub use solve_renderer::*;
pub mod text_renderer;
pub use self::text_renderer::*;

/// A trait to initialise a maze.
pub trait Initializer<R>
where
    R: initialize::Randomizer + Sized,
{
    /// Initialises a maze.
    ///
    /// # Arguments
    /// *  `maze` - The maze to initialise.
    /// *  `rng` - A random number generator.
    /// *  `method` - The initialisation method to use.
    fn initialize(
        &self,
        maze: maze::Maze,
        rng: &mut R,
        method: Methods<R>,
    ) -> maze::Maze;
}

impl<R, T> Initializer<R> for Option<T>
where
    R: initialize::Randomizer + Sized,
    T: Initializer<R>,
{
    fn initialize(
        &self,
        maze: maze::Maze,
        rng: &mut R,
        methods: Methods<R>,
    ) -> maze::Maze {
        if let Some(action) = self {
            action.initialize(maze, rng, methods)
        } else {
            methods.initialize(maze, rng, |_| true)
        }
    }
}

/// A trait to perform post-processing of a maze.
pub trait PostProcessor<R>
where
    R: initialize::Randomizer + Sized,
{
    /// Performs post-processing of a maze.
    ///
    /// # Arguments
    /// *  `maze` - The maze to post-process.
    /// *  `rng` - A random number generator.
    fn post_process(&self, maze: maze::Maze, rng: &mut R) -> maze::Maze;
}

impl<R, T> PostProcessor<R> for Option<T>
where
    R: initialize::Randomizer + Sized,
    T: PostProcessor<R>,
{
    fn post_process(&self, maze: maze::Maze, rng: &mut R) -> maze::Maze {
        if let Some(action) = self {
            action.post_process(maze, rng)
        } else {
            maze
        }
    }
}

pub struct Methods<R>(pub voronoi::initialize::Methods<R>)
where
    R: initialize::Randomizer + Sized;

impl<R> Default for Methods<R>
where
    R: initialize::Randomizer + Sized,
{
    fn default() -> Self {
        Self(voronoi::initialize::Methods::default())
    }
}

impl<R> str::FromStr for Methods<R>
where
    R: initialize::Randomizer + Sized,
{
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut methods = vec![];
        for method in s.split(",") {
            methods.push(method.parse()?)
        }

        Ok(Self(voronoi::initialize::Methods::new(methods)))
    }
}

impl<R> Methods<R>
where
    R: initialize::Randomizer + Sized,
{
    /// Wraps the inner initialiser.
    ///
    /// # Arguments
    /// *  `maze` - The maze to initialise.
    /// *  `rng` - A random number generator.
    /// *  `filter` - An additional filter applied to all methods.
    pub fn initialize<F>(
        self,
        maze: maze::Maze,
        rng: &mut R,
        filter: F,
    ) -> maze::Maze
    where
        F: Fn(matrix::Pos) -> bool,
    {
        let (_, maze) = self.0.initialize(maze, rng, filter);
        maze
    }
}

/// A trait for rendering a maze.
pub trait Renderer {
    /// Applies this action to a maze and SVG group.
    ///
    /// # Arguments
    /// *  `maze` - The maze.
    /// *  `group` - An SVG group.
    fn render(&self, maze: &maze::Maze, group: &mut svg::node::element::Group);
}

impl<T> Renderer for Option<T>
where
    T: Renderer,
{
    fn render(&self, maze: &maze::Maze, group: &mut svg::node::element::Group) {
        if let Some(action) = self {
            action.render(maze, group);
        }
    }
}

/// A type of heat map.
pub enum HeatMapType {
    /// The heat map is generated by traversing vertically.
    Vertical,

    /// The heat map is generated by traversing horisontally.
    Horizontal,

    /// The heat map is generated by travesing from every edge room to the one
    /// on the opposite side.
    Full,
}

impl str::FromStr for HeatMapType {
    type Err = String;

    fn from_str(s: &str) -> Result<HeatMapType, Self::Err> {
        match s {
            "vertical" => Ok(HeatMapType::Vertical),
            "horizontal" => Ok(HeatMapType::Horizontal),
            "full" => Ok(HeatMapType::Full),
            _ => Err(format!("unknown heat map type: {}", s)),
        }
    }
}

impl HeatMapType {
    /// Generates a heat map based on this heat map type.
    ///
    /// # Arguments
    /// * `maze` - The maze for which to generate a heat map.
    pub fn generate(&self, maze: &maze::Maze) -> maze::matrix::Matrix<u32> {
        match *self {
            HeatMapType::Vertical => self.create_heatmap(
                maze,
                (0..maze.width()).map(|col| {
                    (
                        maze::matrix::Pos {
                            col: col as isize,
                            row: 0,
                        },
                        maze::matrix::Pos {
                            col: col as isize,
                            row: maze.height() as isize - 1,
                        },
                    )
                }),
            ),
            HeatMapType::Horizontal => self.create_heatmap(
                maze,
                (0..maze.height()).map(|row| {
                    (
                        maze::matrix::Pos {
                            col: 0,
                            row: row as isize,
                        },
                        maze::matrix::Pos {
                            col: maze.width() as isize - 1,
                            row: row as isize,
                        },
                    )
                }),
            ),
            HeatMapType::Full => self.create_heatmap(
                maze,
                maze.rooms()
                    .positions()
                    .filter(|&pos| pos.col == 0 || pos.row == 0)
                    .map(|pos| {
                        (
                            pos,
                            maze::matrix::Pos {
                                col: maze.width() as isize - 1 - pos.col,
                                row: maze.height() as isize - 1 - pos.row,
                            },
                        )
                    }),
            ),
        }
    }

    /// Generates a heat map for a maze and an iteration of positions.
    ///
    /// # Arguments
    /// * `maze` - The maze for which to generate a heat map.
    /// * `positions` - The positions for which to generate a heat map. These
    ///   will be generated from the heat map type.
    fn create_heatmap<I>(
        &self,
        maze: &maze::Maze,
        positions: I,
    ) -> maze::HeatMap
    where
        I: Iterator<Item = (maze::matrix::Pos, maze::matrix::Pos)>,
    {
        let collected = positions.collect::<Vec<_>>();
        collected
            .chunks(collected.len() / rayon::current_num_threads())
            .collect::<Vec<_>>()
            .par_iter()
            .map(|positions| maze::heatmap(maze, positions.iter().cloned()))
            .reduce(
                || maze::HeatMap::new(maze.width(), maze.height()),
                AddableMatrix::add,
            )
    }
}

/// Draws all rooms of a maze.
///
/// # Arguments
/// * `maze` - The maze to draw.
/// * `colors` - A function determining the colour of a room.
pub fn draw_rooms<F>(maze: &maze::Maze, colors: F) -> svg::node::element::Group
where
    F: Fn(maze::matrix::Pos) -> Color,
{
    let mut group = svg::node::element::Group::new();
    for pos in maze
        .rooms()
        .positions()
        .filter(|&pos| maze.rooms()[pos].visited)
    {
        let color = colors(pos);
        let mut commands = maze
            .walls(pos)
            .iter()
            .enumerate()
            .map(|(i, wall)| {
                let (coords, _) = maze.corners((pos, wall));
                if i == 0 {
                    svg::node::element::path::Command::Move(
                        svg::node::element::path::Position::Absolute,
                        (coords.x, coords.y).into(),
                    )
                } else {
                    svg::node::element::path::Command::Line(
                        svg::node::element::path::Position::Absolute,
                        (coords.x, coords.y).into(),
                    )
                }
            })
            .collect::<Vec<_>>();
        commands.push(svg::node::element::path::Command::Close);

        group.append(
            svg::node::element::Path::new()
                .set("fill", color.to_string())
                .set("fill-opacity", f32::from(color.alpha) / 255.0)
                .set("d", svg::node::element::path::Data::from(commands)),
        );
    }

    group
}
