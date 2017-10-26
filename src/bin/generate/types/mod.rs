use std;

#[cfg(feature = "background")]
use image;

#[cfg(feature = "parallel")]
use rayon::current_num_threads;
#[cfg(feature = "parallel")]
use rayon::prelude::*;

use svg;
use svg::Node;

use labyru;

#[cfg(feature = "parallel")]
use labyru::matrix::AddableMatrix;


pub mod background_action;
pub mod break_action;
pub mod heatmap_action;


/// A trait for actions passed on the command line.
pub trait Action {
    /// Converts a string to an action.
    ///
    /// # Arguments
    /// *  `s` - The string to convert.
    fn from_str(s: &str) -> Result<Self, String>
    where
        Self: std::marker::Sized;

    /// Applies this action to a maze and SVG group.
    ///
    /// # Arguments
    /// *  `maze` - The maze.
    /// *  `group` - An SVG group.
    fn apply(
        self,
        maze: &mut labyru::Maze,
        group: &mut svg::node::element::Group,
    );
}


/// A colour.
#[derive(Clone, Copy, Default)]
pub struct Color {
    // The red component.
    pub red: u8,

    // The green component.
    pub green: u8,

    // The blue component.
    pub blue: u8,

    // The alpha component.
    pub alpha: u8,
}


impl Color {
    /// Converts a string to a colour.
    ///
    /// This method supports colouts on the form `#RRGGBB` and `#RRGGBBAA`,
    /// where `RR`, `GG`, `BB` and `AA` are the red, green, blue and alpha
    // components hex encoded.
    ///
    /// # Arguments
    /// * `s` - The string to convert.
    pub fn from_str(s: &str) -> Result<Color, String> {
        if !s.starts_with('#') || s.len() % 1 == 1 {
            Err(format!("unknown colour value: {}", s))
        } else {
            let data = s.bytes()
                // Skip the initial '#'
                .skip(1)

                // Hex decode and create list
                .map(|c| if c >= '0' as u8 && c <= '9' as u8 {
                    Some(c - '0' as u8)
                } else if c >= 'A' as u8 && c <= 'F' as u8 {
                    Some(c - 'A' as u8 + 10)
                } else if c >= 'a' as u8 && c <= 'f' as u8 {
                    Some(c - 'a' as u8 + 10)
                } else {
                    None
                })
                .collect::<Vec<_>>()

                // Join every byte
                .chunks(2)
                .map(|c| if let (Some(msb), Some(lsb)) = (c[0], c[1]) {
                    Some(msb << 4 | lsb)
                } else {
                    None
                })

                // Ensure all values are valid
                .take_while(|c| c.is_some())
                .map(|c| c.unwrap())
                .collect::<Vec<_>>();

            match data.len() {
                3 => Ok(Color {
                    red: data[0],
                    green: data[1],
                    blue: data[2],
                    alpha: 255,
                }),
                4 => Ok(Color {
                    red: data[1],
                    green: data[2],
                    blue: data[3],
                    alpha: data[0],
                }),
                _ => Err(format!("invalid colour format: {}", s)),
            }
        }
    }

    /// Returns a fully transparent version of this colour.
    pub fn transparent(&self) -> Self {
        Self {
            red: self.red,
            green: self.blue,
            blue: self.blue,
            alpha: 0,
        }
    }

    /// Fades one colour to another.
    ///
    /// # Arguments
    /// * `other` - The other colour.
    /// * `w` - The weight of this colour. If this is `1.0` or greater, `self`
    ///   colour is returned; if this is 0.0 or less, `other` is returned;
    ///   otherwise a linear interpolation between the colours is returned.
    pub fn fade(&self, other: &Self, w: f32) -> Color {
        if w >= 1.0 {
            self.clone()
        } else if w <= 0.0 {
            other.clone()
        } else {
            let n = 1.0 - w;
            Color {
                red: (self.red as f32 * w + other.red as f32 * n) as u8,
                green: (self.green as f32 * w + other.green as f32 * n) as u8,
                blue: (self.blue as f32 * w + other.blue as f32 * n) as u8,
                alpha: (self.alpha as f32 * w + other.alpha as f32 * n) as u8,
            }
        }
    }
}


impl ToString for Color {
    /// Converts a colour to a string.
    ///
    /// This method ignores the alpha component.
    fn to_string(&self) -> String {
        format!("#{:02.X}{:02.X}{:02.X}", self.red, self.green, self.blue)
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


impl HeatMapType {
    /// Converts a string to a heat map type.
    ///
    /// # Arguments
    /// * `s` - The string to convert.
    pub fn from_str(s: &str) -> Result<HeatMapType, String> {
        match s {
            "vertical" => Ok(HeatMapType::Vertical),
            "horizontal" => Ok(HeatMapType::Horizontal),
            "full" => Ok(HeatMapType::Full),
            _ => Err(format!("unknown heat map type: {}", s)),
        }
    }

    /// Generates a heat map based on this heat map type.
    ///
    /// # Arguments
    /// * `maze` - The maze for which to generate a heat map.
    pub fn generate(&self, maze: &labyru::Maze) -> labyru::matrix::Matrix<u32> {
        match *self {
            HeatMapType::Vertical => {
                self.create_heatmap(
                    maze,
                    (0..maze.width()).map(|x| {
                        (
                            (x as isize, 0),
                            (x as isize, maze.height() as isize - 1),
                        )
                    }),
                )
            }
            HeatMapType::Horizontal => {
                self.create_heatmap(
                    maze,
                    (0..maze.height()).map(|y| {
                        (
                            (0, y as isize),
                            (maze.width() as isize - 1, y as isize),
                        )
                    }),
                )
            }
            HeatMapType::Full => {
                self.create_heatmap(
                    maze,
                    maze.rooms()
                        .positions()
                        .filter(|&(x, y)| x == 0 || y == 0)
                        .map(|(x, y)| {
                            ((x, y), (
                                maze.width() as isize - 1 - x,
                                maze.height() as isize - 1 - y,
                            ))
                        }),
                )
            }
        }
    }

    /// Generates a heat map for a maze and an iteration of positions.
    ///
    /// # Arguments
    /// * `maze` - The maze for which to generate a heat map.
    /// * `positions` - The positions for which to generate a heat map. These
    ///   will be generated from the heat map type.
    #[cfg(not(feature = "parallel"))]
    fn create_heatmap<I>(
        &self,
        maze: &labyru::Maze,
        positions: I,
    ) -> labyru::HeatMap
    where
        I: Iterator<Item = (labyru::matrix::Pos, labyru::matrix::Pos)>,
    {
        labyru::heatmap(maze, positions)
    }

    /// Generates a heat map for a maze and an iteration of positions.
    ///
    /// # Arguments
    /// * `maze` - The maze for which to generate a heat map.
    /// * `positions` - The positions for which to generate a heat map. These
    ///   will be generated from the heat map type.
    #[cfg(feature = "parallel")]
    fn create_heatmap<I>(
        &self,
        maze: &labyru::Maze,
        positions: I,
    ) -> labyru::HeatMap
    where
        I: Iterator<Item = (labyru::matrix::Pos, labyru::matrix::Pos)>,
    {
        let collected = positions.collect::<Vec<_>>();
        collected
            .chunks(collected.len() / current_num_threads())
            .collect::<Vec<_>>()
            .par_iter()
            .map(|positions| {
                labyru::heatmap(maze, positions.iter().map(|p| *p))
            })
            .reduce(|| labyru::HeatMap::new(maze.width(), maze.height()), |acc,
             o| {
                acc.add(o)
            })
    }
}


/// Draws all rooms of a maze.
///
/// # Arguments
/// * `maze` - The maze to draw.
/// * `colors` - A function determining the colour of a room.
pub fn draw_rooms<F>(
    maze: &labyru::Maze,
    colors: F,
) -> svg::node::element::Group
where
    F: Fn(labyru::matrix::Pos) -> Color,
{
    let mut group = svg::node::element::Group::new();
    for pos in maze.rooms().positions().filter(
        |pos| maze.rooms()[*pos].visited,
    )
    {
        let color = colors(pos);
        let mut commands = maze.walls(pos)
            .iter()
            .enumerate()
            .map(|(i, wall)| {
                let (coords, _) = maze.corners((pos, wall));
                if i == 0 {
                    svg::node::element::path::Command::Move(
                        svg::node::element::path::Position::Absolute,
                        coords.into(),
                    )
                } else {
                    svg::node::element::path::Command::Line(
                        svg::node::element::path::Position::Absolute,
                        coords.into(),
                    )
                }
            })
            .collect::<Vec<_>>();
        commands.push(svg::node::element::path::Command::Close);

        group.append(
            svg::node::element::Path::new()
                .set("fill", color.to_string())
                .set("fill-opacity", (color.alpha as f32 / 255.0))
                .set("d", svg::node::element::path::Data::from(commands)),
        );
    }

    group
}


/// Converts an image to a matrix by calling an update function with a pixel
/// and its corresponding matrix position.
///
/// # Arguments
/// *  `image` - The image to convert.
/// *  `maze` - A template maze. This is used to determine which matrix
///    position a pixel corresponds to, and to determine the dimensions of the
///    matrix.
/// *  `update` - The update function.
#[cfg(feature = "background")]
pub fn image_to_matrix<U, T>(
    image: image::RgbImage,
    maze: &labyru::Maze,
    update: U,
) -> labyru::matrix::Matrix<T>
where
    U: Fn(&mut labyru::matrix::Matrix<T>,
       labyru::matrix::Pos,
       &image::Rgb<u8>),
    T: Copy + Default,
{
    let (left, top, width, height) = maze.viewbox();
    let (cols, rows) = image.dimensions();
    image
        .enumerate_pixels()
        .fold(
            labyru::matrix::Matrix::<T>::new(maze.width(), maze.height()),
            |mut matrix, (x, y, pixel)| {
                let physical_pos = (
                    left + width * (x as f32 / cols as f32),
                    top + height * (y as f32 / rows as f32),
                );
                let pos = maze.room_at(physical_pos);
                update(&mut matrix, pos, pixel);
                matrix
            },
        )
}
