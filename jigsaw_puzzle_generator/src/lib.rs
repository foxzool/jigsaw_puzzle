//! The Jigsaw Puzzle library creates SVG paths which can be used to cut out puzzle pieces from a
//! given rectangular image. It provides three public functions:
//!
//! - [`build_jigsaw_pieces`] returns the paths from a given number of pieces in a column and a
//!     row. This is the function you normally want to use
//! - [`generate_columns_rows_numbers`] returns an ideal distribution of pieces on the x- and y-axes
//!     for a given total number of pieces
//! - [`round`] is a util function which approximately rounds a f32 value to two decimal places

use anyhow::{anyhow, Result};
use bezier_rs::{Bezier, BezierHandles, Identifier, Subpath};
use glam::DVec2;
use image::{DynamicImage, GenericImageView, Rgba, RgbaImage};

use log::{debug, info};
use rayon::iter::ParallelIterator;
use std::vec;

pub use image;
pub use imageproc;

const DEFAULT_TAB_SIZE: f32 = 20.0;
const DEFAULT_JITTER: f32 = 5.0;

const MAX_WIDTH: u32 = 1920;
const MAX_HEIGHT: u32 = 1200;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum GameMode {
    #[default]
    Classic,
    Square,
}

/// A segment of an indented puzzle piece edge. A segment is described by a cubic Bézier curve,
/// which includes a starting point, an end point and two control points. Three segments make up a
/// piece's edge.
#[derive(Clone, PartialEq, Debug)]
pub struct IndentationSegment {
    /// Starting point of the segment
    pub starting_point: (f32, f32),
    /// End point of the segment
    pub end_point: (f32, f32),
    /// The cubic Bézier curve's first control point
    pub control_point_1: (f32, f32),
    /// The cubic Bézier curve's second control point
    pub control_point_2: (f32, f32),
}

impl IndentationSegment {
    pub fn to_bezier(&self, reverse: bool) -> Bezier {
        if reverse {
            Bezier::from_cubic_coordinates(
                self.end_point.0 as f64,
                self.end_point.1 as f64,
                self.control_point_2.0 as f64,
                self.control_point_2.1 as f64,
                self.control_point_1.0 as f64,
                self.control_point_1.1 as f64,
                self.starting_point.0 as f64,
                self.starting_point.1 as f64,
            )
        } else {
            Bezier::from_cubic_coordinates(
                self.starting_point.0 as f64,
                self.starting_point.1 as f64,
                self.control_point_1.0 as f64,
                self.control_point_1.1 as f64,
                self.control_point_2.0 as f64,
                self.control_point_2.1 as f64,
                self.end_point.0 as f64,
                self.end_point.1 as f64,
            )
        }
    }
}

#[derive(Clone, PartialEq, Debug)]
/// An indented puzzle piece edge. An edge is decribe via three distinct cubic Bézier curves (the
/// "segments")
pub struct IndentedEdge {
    /// Describes the left half for a horizontal edge, the upper half for a vertical edge
    pub first_segment: IndentationSegment,
    /// Describes the form of the tab
    pub middle_segment: IndentationSegment,
    /// Describes the right half for a horizontal edge, the lower half for a vertical edge
    pub last_segment: IndentationSegment,
}

#[allow(dead_code)]
const RED_COLOR: Rgba<u8> = Rgba([255, 0, 0, 255]);
#[allow(dead_code)]
const BLACK_COLOR: Rgba<u8> = Rgba([0, 0, 0, 255]);
#[allow(dead_code)]
const WHITE_COLOR: Rgba<u8> = Rgba([255, 255, 255, 255]);
#[allow(dead_code)]
const YELLOW_COLOR: Rgba<u8> = Rgba([255, 255, 0, 255]);

impl IndentedEdge {
    /// Creates a new indented edge
    pub fn new(
        starting_point: (f32, f32),
        end_point: (f32, f32),
        generator: &mut EdgeContourGenerator,
    ) -> Self {
        generator.create(starting_point, end_point)
    }

    pub fn to_beziers(&self, reverse: bool) -> Vec<Bezier> {
        if reverse {
            vec![
                self.last_segment.to_bezier(reverse),
                self.middle_segment.to_bezier(reverse),
                self.first_segment.to_bezier(reverse),
            ]
        } else {
            vec![
                self.first_segment.to_bezier(reverse),
                self.middle_segment.to_bezier(reverse),
                self.last_segment.to_bezier(reverse),
            ]
        }
    }
}

/// Provides the means to generate [`IndentedEdge`]s
#[derive(Debug, Clone)]
pub struct EdgeContourGenerator {
    /// The baseline width of a puzzle piece
    piece_width: f32,
    /// The baseline height of a puzzle piece
    piece_height: f32,
    /// The tab size factor
    tab_size: f32,
    /// The "jitter" factor. A bigger number makes the puzzle pieces more asymmetrical
    jitter: f32,
    /// Seed for random values. Increased by 1 after each iteration.
    seed: usize,
    /// Flipped tab
    flipped: bool,
    /// Random value based on the seed and the predefined jitter value.
    a: f32,
    b: f32,
    c: f32,
    d: f32,
    e: f32,
}

impl EdgeContourGenerator {
    /// Creates a new [`EdgeContourGenerator`] instance after making sure that the optionally
    /// provided `tab_size`, `jitter` and `seed` values are in the allowed ranges
    pub fn new(
        piece_width: f32,
        piece_height: f32,
        tab_size: Option<f32>,
        jitter: Option<f32>,
        seed: Option<usize>,
    ) -> EdgeContourGenerator {
        let tab_size = tab_size.unwrap_or(DEFAULT_TAB_SIZE) / 200.0;
        assert!((0.05..=0.15).contains(&tab_size));
        let jitter = jitter.unwrap_or(DEFAULT_JITTER) / 100.0;
        assert!((0.0..=0.13).contains(&jitter));
        let seed = seed.unwrap_or(0);
        let e = Self::uniform(-jitter, jitter, seed + 1);
        let (seed, flipped, a, b, c, d, e) = Self::dice(e, false, seed + 2, jitter);
        EdgeContourGenerator {
            piece_width,
            piece_height,
            tab_size,
            jitter,
            seed,
            flipped,
            a,
            b,
            c,
            d,
            e,
        }
    }

    /// Normalises the seed value on a scale between 0 and 1
    fn normalise(seed: usize) -> f32 {
        let x = f32::sin(seed as f32) * 10000.0;
        x - f32::floor(x)
    }

    /// Returns a statistically evenly distributed value between a `min` and a `max` value
    fn uniform(min: f32, max: f32, seed: usize) -> f32 {
        min + Self::normalise(seed) * (max - min)
    }

    /// Returns `true` if the given value is greater than 0.5 after being normalised on a scale
    /// between 0.0 and 1.0. I.e. the chances should be approximately 50% for the result to be
    /// `true`.
    fn rbool(seed: usize) -> bool {
        Self::normalise(seed) > 0.5
    }

    /// Recomputes the factors influencing the form of the edge
    fn dice(
        e: f32,
        flipped: bool,
        seed: usize,
        jitter: f32,
    ) -> (usize, bool, f32, f32, f32, f32, f32) {
        let new_flipped = Self::rbool(seed);
        let a = if new_flipped == flipped { -e } else { e };
        let b = Self::uniform(-jitter, jitter, seed + 2);
        let c = Self::uniform(-jitter, jitter, seed + 3);
        let d = Self::uniform(-jitter, jitter, seed + 4);
        let e = Self::uniform(-jitter, jitter, seed + 5);
        (seed + 6, new_flipped, a, b, c, d, e)
    }

    /// Computes the position of a point on an axis along the piece's edge
    fn longitudinal_position(coeff: f32, offset: f32, length: f32) -> f32 {
        round(offset + coeff * length)
    }

    /// Computes the position of a point on an axis transverse to the piece's edge
    fn transverse_position(coeff: f32, offset: f32, length: f32, flipped: bool) -> f32 {
        round(offset + coeff * length * if flipped { -1.0 } else { 1.0 })
    }

    /// Gets the coordinates of a point in a cubic Bézier curve relative to a starting point, the
    /// length and the side of the edge (horizontal, vertical) and finally two coefficients
    /// which designate the offset of the respective points on the longitudinal (`l_coeff`) and the
    /// transverse (`t_coeff`) axes.
    fn coords(
        &self,
        l_coeff: f32,
        t_coeff: f32,
        starting_point: (f32, f32),
        vertical: bool,
    ) -> (f32, f32) {
        let pos_1 = Self::longitudinal_position(
            l_coeff,
            if vertical {
                starting_point.1
            } else {
                starting_point.0
            },
            if vertical {
                self.piece_height
            } else {
                self.piece_width
            },
        );
        let pos_2 = Self::transverse_position(
            t_coeff,
            if vertical {
                starting_point.0
            } else {
                starting_point.1
            },
            if vertical {
                self.piece_width
            } else {
                self.piece_height
            },
            self.flipped,
        );
        if vertical {
            (pos_2, pos_1)
        } else {
            (pos_1, pos_2)
        }
    }

    /// Coordinates of the first segment's end point
    fn ep1(&self, starting_point: (f32, f32), vertical: bool) -> (f32, f32) {
        self.coords(
            0.5 - self.tab_size + self.b,
            self.tab_size + self.c,
            starting_point,
            vertical,
        )
    }

    /// Coordinates of the first segment's first control point
    fn cp1_1(&self, starting_point: (f32, f32), vertical: bool) -> (f32, f32) {
        self.coords(0.2, self.a, starting_point, vertical)
    }

    /// Coordinates of the first segment's second control point
    fn cp1_2(&self, starting_point: (f32, f32), vertical: bool) -> (f32, f32) {
        self.coords(
            0.5 + self.b + self.d,
            -self.tab_size + self.c,
            starting_point,
            vertical,
        )
    }

    /// Coordinates of the second segment's end point
    fn ep2(&self, starting_point: (f32, f32), vertical: bool) -> (f32, f32) {
        self.coords(
            0.5 + self.tab_size + self.b,
            self.tab_size + self.c,
            starting_point,
            vertical,
        )
    }

    /// Coordinates of the second segment's first control point
    fn cp2_1(&self, starting_point: (f32, f32), vertical: bool) -> (f32, f32) {
        self.coords(
            0.5 - 2.0 * self.tab_size + self.b - self.d,
            3.0 * self.tab_size + self.c,
            starting_point,
            vertical,
        )
    }

    /// Coordinates of the second segment's second control point
    fn cp2_2(&self, starting_point: (f32, f32), vertical: bool) -> (f32, f32) {
        self.coords(
            0.5 + 2.0 * self.tab_size + self.b - self.d,
            3.0 * self.tab_size + self.c,
            starting_point,
            vertical,
        )
    }

    /// Coordinates of the third segment's first control point
    fn cp3_1(&self, starting_point: (f32, f32), vertical: bool) -> (f32, f32) {
        self.coords(
            0.5 + self.b + self.d,
            -self.tab_size + self.b + self.d,
            starting_point,
            vertical,
        )
    }

    /// Coordinates of the third segment's second control point
    fn cp3_2(&self, starting_point: (f32, f32), vertical: bool) -> (f32, f32) {
        self.coords(0.8, self.e, starting_point, vertical)
    }

    /// Returns a new [`IndentedEdge`] from a given starting and end point
    pub fn create(&mut self, starting_point: (f32, f32), end_point: (f32, f32)) -> IndentedEdge {
        let vertical = (end_point.0 - starting_point.0).abs() < 1.0;
        let first_segment = IndentationSegment {
            starting_point,
            end_point: self.ep1(starting_point, vertical),
            control_point_1: self.cp1_1(starting_point, vertical),
            control_point_2: self.cp1_2(starting_point, vertical),
        };
        let middle_segment = IndentationSegment {
            starting_point: self.ep1(starting_point, vertical),
            end_point: self.ep2(starting_point, vertical),
            control_point_1: self.cp2_1(starting_point, vertical),
            control_point_2: self.cp2_2(starting_point, vertical),
        };
        let last_segment = IndentationSegment {
            starting_point: self.ep2(starting_point, vertical),
            end_point,
            control_point_1: self.cp3_1(starting_point, vertical),
            control_point_2: self.cp3_2(starting_point, vertical),
        };
        let indented_edge = IndentedEdge {
            first_segment,
            middle_segment,
            last_segment,
        };
        (
            self.seed,
            self.flipped,
            self.a,
            self.b,
            self.c,
            self.d,
            self.e,
        ) = Self::dice(self.e, false, self.seed + 2, self.jitter);
        indented_edge
    }
}

#[derive(Clone, PartialEq, Debug)]
/// A puzzle piece edge which is at the same time a part of the puzzle's border and therefore forms
/// a straight line
pub struct StraightEdge {
    pub starting_point: (f32, f32),
    pub end_point: (f32, f32),
}

impl StraightEdge {
    pub fn to_beziers(&self, reverse: bool) -> Vec<Bezier> {
        if reverse {
            vec![Bezier::from_linear_coordinates(
                self.end_point.0 as f64,
                self.end_point.1 as f64,
                self.starting_point.0 as f64,
                self.starting_point.1 as f64,
            )]
        } else {
            vec![Bezier::from_linear_coordinates(
                self.starting_point.0 as f64,
                self.starting_point.1 as f64,
                self.end_point.0 as f64,
                self.end_point.1 as f64,
            )]
        }
    }
}

#[derive(Clone, PartialEq, Debug)]
/// A border of a puzzle piece. Can be either an `StraightEdge` (no adjacent other piece) or an
/// `IndentedEdge`
pub enum Edge {
    IndentedEdge(IndentedEdge),
    StraightEdge(StraightEdge),
}

impl Edge {
    pub fn to_beziers(&self, reverse: bool) -> Vec<Bezier> {
        match self {
            Edge::IndentedEdge(ie) => ie.to_beziers(reverse),
            Edge::StraightEdge(oe) => oe.to_beziers(reverse),
        }
    }
}

/// Divides the axis into `pieces` of equal length. Returns the starting point of each piece,
/// i.e. the x coordinate on the left of the piece for horizontal lines, and the y coordinate on
/// the top of the piece for vertical lines, and the length of the piece.
fn divide_axis(length: f32, piece_num: usize) -> (Vec<f32>, f32) {
    let piece_length = round(length / piece_num as f32);
    (
        (0..piece_num)
            .map(|s| round(s as f32 * piece_length))
            .collect::<Vec<f32>>(),
        piece_length,
    )
}

/// Rounds a given rational number to two decimal places
pub fn round(x: f32) -> f32 {
    (x * 100.0).round() / 100.0
}

/// Returns the indices of the top, right, bottom and left edge from a given `position` of the
/// piece in a one-dimensional list of all pieces in the jigsaw puzzle. The returned indices are
/// used to get the SVG paths for the edges from two lists of all vertical and horizontal edges.
fn get_border_indices(position: usize, number_of_columns: usize) -> (usize, usize, usize, usize) {
    let row_ind = position / number_of_columns;
    (
        position,
        position + 1 + row_ind,
        position + number_of_columns,
        position + row_ind,
    )
}

/// Returns the position of a given segment's end
fn end_point_pos(ind: usize, segments: &[f32], fallback: f32) -> f32 {
    if ind < (segments.len() - 1) {
        segments[ind + 1]
    } else {
        fallback
    }
}

/// Returns all divisor pairs for a given number
fn find_divisors(num: usize) -> Vec<(usize, usize)> {
    let mut i = 1;
    let mut divisor_pairs = vec![];
    loop {
        if i * i > num {
            break;
        } else if num % i == 0 {
            divisor_pairs.push((i, num / i));
        }
        i += 1;
    }
    let mut mirrored = divisor_pairs
        .iter()
        .filter(|(a, b)| a != b)
        .map(|(a, b)| (*b, *a))
        .collect::<Vec<(usize, usize)>>();
    mirrored.reverse();
    divisor_pairs.append(&mut mirrored);
    divisor_pairs
}

/// Returns the visually most appealing piece aspect ratio, i.e. a square one (equal width and
/// height) or, if that's not possible , a "landscape" format as square as possible.
fn optimal_aspect_ratio(
    possible_dimensions: Vec<(usize, usize)>,
    image_width: f32,
    image_height: f32,
) -> Result<(usize, usize)> {
    let mut width_height_diff = f32::MAX;
    let mut number_of_pieces = *possible_dimensions
        .first()
        .ok_or_else(|| anyhow!("No possible dimensions found"))?;
    for (x, y) in possible_dimensions {
        let width = image_width / x as f32;
        let height = image_height / y as f32;
        let new_width_height_diff = (width - height).abs();
        if new_width_height_diff < 1. {
            return Ok((x, y));
        }
        if width_height_diff >= new_width_height_diff {
            width_height_diff = new_width_height_diff;
            number_of_pieces = (x, y);
        } else {
            return Ok(number_of_pieces);
        }
    }
    Ok(number_of_pieces)
}

/// Returns the visually most appealing numbers of pieces in one column and one row based on a
/// given number of pieces
pub fn generate_columns_rows_numbers(
    image_width: f32,
    image_height: f32,
    number_of_pieces: usize,
) -> Result<(usize, usize)> {
    let divisor_pairs = find_divisors(number_of_pieces);
    optimal_aspect_ratio(divisor_pairs, image_width, image_height)
}

/// A jigsaw pieces generator
///
/// Returns list on how to cut jigsaw puzzle pieces from an image of a given width and
/// height and the number of pieces in a row and a column as an optional the tab size, a "jitter"
/// factor and an initial seed value.
///
/// The `tab_size` argument defines the size of the pieces' tabs. It can be any number from `10.0` to `30.0` and defaults to `20.0`
///
/// `jitter` can be a number between 0.0 and 13.0. The bigger the number, the more asymmetrical are
/// the puzzle pieces. Defaults to `0.0` (symmetrical).
///
/// `seed` provides the initial "randomness" when creating the contours of the puzzle pieces. Same
/// seed values for images with same dimensions and same number of pieces lead to same SVG paths.
#[derive(Debug)]
pub struct JigsawGenerator {
    /// The original image from which the jigsaw puzzle pieces will be generated.
    origin_image: DynamicImage,
    /// The number of pieces in a column.
    pieces_in_column: usize,
    /// The number of pieces in a row.
    pieces_in_row: usize,
    /// Optional size of the tabs on the puzzle pieces.
    tab_size: Option<f32>,
    /// Optional jitter factor to introduce asymmetry in the puzzle pieces.
    jitter: Option<f32>,
    /// Optional seed value for randomness in generating the puzzle pieces.
    seed: Option<usize>,
}

impl JigsawGenerator {
    pub fn new(origin_image: DynamicImage, pieces_in_column: usize, pieces_in_row: usize) -> Self {
        JigsawGenerator {
            origin_image,
            pieces_in_column,
            pieces_in_row,
            tab_size: None,
            jitter: None,
            seed: None,
        }
    }

    pub fn from_rgba8(
        width: u32,
        height: u32,
        image_bytes: &[u8],
        pieces_in_column: usize,
        pieces_in_row: usize,
    ) -> Result<Self> {
        let origin_image = DynamicImage::ImageRgba8(
            RgbaImage::from_raw(width, height, image_bytes.to_vec())
                .ok_or_else(|| anyhow!("Failed to create image from raw bytes"))?,
        );
        Ok(JigsawGenerator::new(
            origin_image,
            pieces_in_column,
            pieces_in_row,
        ))
    }

    /// Creates a new `JigsawGenerator` instance from an image file at the given `image_path`
    /// with a given number of pieces in a column and a row.
    pub fn from_path(
        image_path: &str,
        pieces_in_column: usize,
        pieces_in_row: usize,
    ) -> Result<Self> {
        let origin_image = image::open(image_path)?;
        info!(
            "loaded image from {} with dimensions {}x{}",
            image_path,
            origin_image.width(),
            origin_image.height()
        );
        Ok(JigsawGenerator {
            origin_image,
            pieces_in_column,
            pieces_in_row,
            tab_size: None,
            jitter: None,
            seed: None,
        })
    }

    pub fn tab_size(mut self, tab_size: f32) -> Self {
        self.tab_size = Some(tab_size);
        self
    }

    pub fn jitter(mut self, jitter: f32) -> Self {
        self.jitter = Some(jitter);
        self
    }

    pub fn seed(mut self, seed: usize) -> Self {
        self.seed = Some(seed);
        self
    }

    pub fn origin_image(&self) -> &DynamicImage {
        &self.origin_image
    }

    pub fn pieces_in_column(&self) -> usize {
        self.pieces_in_column
    }

    pub fn pieces_in_row(&self) -> usize {
        self.pieces_in_row
    }

    pub fn pieces_count(&self) -> usize {
        self.pieces_in_column * self.pieces_in_row
    }

    pub fn generate(&self, _game_mode: GameMode, resize: bool) -> Result<JigsawTemplate> {
        let target_image = if resize {
            scale_image(&self.origin_image)
        } else {
            self.origin_image.clone()
        };
        let (target_image_width, target_image_height) = target_image.dimensions();
        info!(
            "start processing image with {}x{}",
            target_image_width, target_image_height
        );
        let image_width = target_image_width as f32;
        let image_height = target_image_height as f32;
        let pieces_in_column = self.pieces_in_column;
        let pieces_in_row = self.pieces_in_row;
        let (starting_points_x, piece_width) = divide_axis(image_width, pieces_in_column);
        let (starting_points_y, piece_height) = divide_axis(image_height, pieces_in_row);
        let mut contour_gen = EdgeContourGenerator::new(
            piece_width,
            piece_height,
            self.tab_size,
            self.jitter,
            self.seed,
        );
        let mut vertical_edges = vec![];
        let mut horizontal_edges = vec![];
        let mut top_border = true;
        for index_y in 0..starting_points_y.len() {
            let mut left_border = true;
            for index_x in 0..starting_points_x.len() {
                horizontal_edges.push(if top_border {
                    Edge::StraightEdge(StraightEdge {
                        starting_point: (starting_points_x[index_x], 0.0),
                        end_point: (end_point_pos(index_x, &starting_points_x, image_width), 0.0),
                    })
                } else {
                    Edge::IndentedEdge(IndentedEdge::new(
                        (starting_points_x[index_x], starting_points_y[index_y]),
                        (
                            end_point_pos(index_x, &starting_points_x, image_width),
                            starting_points_y[index_y],
                        ),
                        &mut contour_gen,
                    ))
                });
                vertical_edges.push(if left_border {
                    Edge::StraightEdge(StraightEdge {
                        starting_point: (0.0, starting_points_y[index_y]),
                        end_point: (
                            0.0,
                            end_point_pos(index_y, &starting_points_y, image_height),
                        ),
                    })
                } else {
                    Edge::IndentedEdge(IndentedEdge::new(
                        (starting_points_x[index_x], starting_points_y[index_y]),
                        (
                            starting_points_x[index_x],
                            end_point_pos(index_y, &starting_points_y, image_height),
                        ),
                        &mut contour_gen,
                    ))
                });
                left_border = false;
            }
            top_border = false;
            // Draw right outer edge
            vertical_edges.push(Edge::StraightEdge(StraightEdge {
                starting_point: (image_width, starting_points_y[index_y]),
                end_point: (
                    image_width,
                    end_point_pos(index_y, &starting_points_y, image_height),
                ),
            }));
        }

        // Draw bottom outer edges
        for index_x in 0..starting_points_x.len() {
            horizontal_edges.push(Edge::StraightEdge(StraightEdge {
                starting_point: (starting_points_x[index_x], image_height),
                end_point: (
                    end_point_pos(index_x, &starting_points_x, image_width),
                    image_height,
                ),
            }))
        }

        let mut pieces = vec![];
        let mut i = 0;
        for y in starting_points_y.iter() {
            for x in starting_points_x.iter() {
                debug!("starting process piece {i}");
                let (top_index, right_index, bottom_index, left_index) =
                    get_border_indices(i, pieces_in_column);
                let piece = JigsawPiece::new(
                    i,
                    (*x, *y),
                    target_image.dimensions(),
                    (piece_width, piece_height),
                    horizontal_edges[top_index].clone(),
                    vertical_edges[right_index].clone(),
                    horizontal_edges[bottom_index].clone(),
                    vertical_edges[left_index].clone(),
                )?;

                debug!("calc beziers end {}", i);

                // draw debug line
                // piece.draw_debug_line(&mut scaled_image);

                pieces.push(piece);
                i += 1;
            }
        }

        Ok(JigsawTemplate {
            pieces,
            origin_image: target_image,
            piece_dimensions: (piece_width, piece_height),
            number_of_pieces: (pieces_in_column, pieces_in_row),
        })
    }
}

#[derive(Debug, Clone)]
pub struct JigsawTemplate {
    /// The generated jigsaw puzzle pieces
    pub pieces: Vec<JigsawPiece>,
    /// The original image from which the jigsaw puzzle pieces will be generated.
    pub origin_image: DynamicImage,
    /// The dimensions (width, length) in pixel
    pub piece_dimensions: (f32, f32),
    /// The number of pieces in the x- and the y-axis
    pub number_of_pieces: (usize, usize),
}

/// Scales the given image to fit within the maximum width and height constraints.
/// If the image dimensions exceed the maximum allowed dimensions, it scales the image down
/// while maintaining the aspect ratio. Otherwise, it returns the original image.
///
/// # Arguments
///
/// * `image` - A reference to the `DynamicImage` that needs to be scaled.
///
/// # Returns
///
/// * `RgbaImage` - The scaled image as an `RgbaImage`.
fn scale_image(image: &DynamicImage) -> DynamicImage {
    let (width, height) = image.dimensions();
    let scale = if width > MAX_WIDTH || height > MAX_HEIGHT {
        let scale_x = MAX_WIDTH as f32 / width as f32;
        let scale_y = MAX_HEIGHT as f32 / height as f32;
        scale_x.min(scale_y)
    } else {
        1.0
    };
    if scale < 1.0 {
        image.resize(
            (width as f32 * scale) as u32,
            (height as f32 * scale) as u32,
            image::imageops::FilterType::Lanczos3,
        )
    } else {
        image.clone()
    }
}

#[derive(Debug, Clone)]
pub struct JigsawPiece {
    pub index: usize,
    pub start_point: (f32, f32),
    pub subpath: Subpath<PuzzleId>,
    pub width: f32,
    pub height: f32,
    pub top_left_x: u32,
    pub top_left_y: u32,
    pub crop_width: u32,
    pub crop_height: u32,
    pub top_edge: Edge,
    pub right_edge: Edge,
    pub bottom_edge: Edge,
    pub left_edge: Edge,
}

impl JigsawPiece {
    pub fn new(
        index: usize,
        start_point: (f32, f32),
        origin_image_size: (u32, u32),
        piece_size: (f32, f32),
        top_edge: Edge,
        right_edge: Edge,
        bottom_edge: Edge,
        left_edge: Edge,
    ) -> Result<Self> {
        let top_beziers = top_edge.to_beziers(false);
        let right_beziers = right_edge.to_beziers(false);
        let bottom_beziers = bottom_edge.to_beziers(true);
        let left_beziers = left_edge.to_beziers(true);
        let beziers: Vec<_> = vec![top_beziers, right_beziers, bottom_beziers, left_beziers]
            .into_iter()
            .flatten()
            .collect();
        let subpath: Subpath<PuzzleId> = Subpath::from_beziers(&beziers, true);
        let [box_min, box_max] = subpath
            .bounding_box()
            .ok_or(anyhow!("No bounding box found"))?;

        let (image_width, image_height) = (origin_image_size.0, origin_image_size.1);
        let (piece_width, piece_height) = (piece_size.0, piece_size.1);
        let piece_width_offset = piece_width * 0.01;
        let piece_height_offset = piece_height * 0.01;
        let top_left_x = (box_min.x as f32 - piece_width_offset).max(0.0) as u32;
        let top_left_y = (box_min.y as f32 - piece_height_offset).max(0.0) as u32;
        let mut crop_width = (box_max.x as f32 - box_min.x as f32 + 2.0 * piece_width_offset)
            .max(piece_width) as u32;
        let mut crop_height = (box_max.y as f32 - box_min.y as f32 + 2.0 * piece_height_offset)
            .max(piece_height) as u32;
        if top_left_x + crop_width > image_width {
            crop_width = image_width - top_left_x;
        }
        if top_left_y + crop_height > image_height {
            crop_height = image_height - top_left_y;
        }

        Ok(JigsawPiece {
            index,
            start_point,
            subpath,
            width: piece_width,
            height: piece_height,
            top_left_x,
            top_left_y,
            crop_width,
            crop_height,
            top_edge,
            right_edge,
            bottom_edge,
            left_edge,
        })
    }

    pub fn calc_offset(&self) -> (f32, f32) {
        let x = self.start_point.0 - self.top_left_x as f32;
        let y = self.start_point.1 - self.top_left_y as f32;
        (x, y)
    }

    pub fn crop(&self, image: &DynamicImage) -> DynamicImage {
        debug!("start crop piece {} image", self.index);
        let mut piece_image = image
            .view(
                self.top_left_x,
                self.top_left_y,
                self.crop_width,
                self.crop_height,
            )
            .to_image();

        piece_image
            .par_enumerate_pixels_mut()
            .for_each(|(x, y, pixel)| {
                let point = DVec2::new(
                    self.top_left_x as f64 + x as f64,
                    self.top_left_y as f64 + y as f64,
                );
                if !self.contains(point) {
                    *pixel = Rgba([0, 0, 0, 0])
                }
            });

        self.draw_bezier(&mut piece_image, WHITE_COLOR);

        piece_image.into()
    }

    /// Fills the not transparent parts of the image with white color
    pub fn fill_white(&self, image: &DynamicImage) -> DynamicImage {
        let mut white_image = image.to_rgba8();
        white_image
            .par_enumerate_pixels_mut()
            .for_each(|(_, _, pixel)| {
                if pixel.0[3] != 0 {
                    *pixel = WHITE_COLOR;
                }
            });

        white_image.into()
    }

    fn draw_bezier(&self, image: &mut RgbaImage, color: Rgba<u8>) {
        let top_left_x = self.top_left_x as f64;
        let top_left_y = self.top_left_y as f64;
        let top_left = DVec2::new(top_left_x, top_left_y);
        for path in self.subpath.iter() {
            match path.handles {
                BezierHandles::Linear => {
                    let start = path.start - top_left - 1.0;
                    let end = path.end - top_left - 1.0;

                    imageproc::drawing::draw_line_segment_mut(
                        image,
                        (start.x.max(0.0) as f32, start.y.max(0.0) as f32),
                        (end.x.max(0.0) as f32, end.y.max(0.0) as f32),
                        color,
                    );
                }
                BezierHandles::Quadratic { .. } => {}
                BezierHandles::Cubic {
                    handle_start,
                    handle_end,
                } => {
                    let start = (path.start.x - top_left_x, path.start.y - top_left_y);
                    let end = (path.end.x - top_left_x, path.end.y - top_left_y);
                    let handle_start = (handle_start.x - top_left_x, handle_start.y - top_left_y);
                    let handle_end = (handle_end.x - top_left_x, handle_end.y - top_left_y);

                    imageproc::drawing::draw_cubic_bezier_curve_mut(
                        image,
                        (start.0 as f32, start.1 as f32),
                        (end.0 as f32, end.1 as f32),
                        (handle_start.0 as f32, handle_start.1 as f32),
                        (handle_end.0 as f32, handle_end.1 as f32),
                        color,
                    );
                }
            }
        }
    }

    pub fn is_on_the_left_side(
        &self,
        other: &JigsawPiece,
        self_loc: (f32, f32),
        other_loc: (f32, f32),
    ) -> bool {
        if (self_loc.0 + self.width - other_loc.0).abs() < COMPARE_THRESHOLD
            && (self_loc.1 - other_loc.1).abs() < COMPARE_THRESHOLD
        {
            self.on_the_left_side(other)
        } else {
            false
        }
    }

    pub fn on_the_left_side(&self, other: &JigsawPiece) -> bool {
        self.right_edge == other.left_edge
    }

    pub fn is_on_the_right_side(
        &self,
        other: &JigsawPiece,
        self_loc: (f32, f32),
        other_loc: (f32, f32),
    ) -> bool {
        if (other_loc.0 + other.width - self_loc.0).abs() < COMPARE_THRESHOLD
            && (self_loc.1 - other_loc.1).abs() < COMPARE_THRESHOLD
        {
            self.on_the_right_side(other)
        } else {
            false
        }
    }

    pub fn on_the_right_side(&self, other: &JigsawPiece) -> bool {
        self.left_edge == other.right_edge
    }

    pub fn is_on_the_top_side(
        &self,
        other: &JigsawPiece,
        self_loc: (f32, f32),
        other_loc: (f32, f32),
    ) -> bool {
        if (other_loc.1 + other.height - self_loc.1).abs() < COMPARE_THRESHOLD
            && (self_loc.0 - other_loc.0).abs() < COMPARE_THRESHOLD
        {
            self.on_the_top_side(other)
        } else {
            false
        }
    }

    pub fn on_the_top_side(&self, other: &JigsawPiece) -> bool {
        self.bottom_edge == other.top_edge
    }

    pub fn is_on_the_bottom_side(
        &self,
        other: &JigsawPiece,
        self_loc: (f32, f32),
        other_loc: (f32, f32),
    ) -> bool {
        if (other_loc.1 - other.height - self_loc.1).abs() < COMPARE_THRESHOLD
            && (self_loc.0 - other_loc.0).abs() < COMPARE_THRESHOLD
        {
            self.on_the_bottom_side(other)
        } else {
            false
        }
    }

    pub fn on_the_bottom_side(&self, other: &JigsawPiece) -> bool {
        self.top_edge == other.bottom_edge
    }

    pub fn beside(&self, other: &JigsawPiece) -> bool {
        self.on_the_top_side(other)
            || self.on_the_bottom_side(other)
            || self.on_the_left_side(other)
            || self.on_the_right_side(other)
    }

    pub fn is_edge(&self) -> bool {
        matches!(self.top_edge, Edge::StraightEdge(_))
            || matches!(self.right_edge, Edge::StraightEdge(_))
            || matches!(self.bottom_edge, Edge::StraightEdge(_))
            || matches!(self.left_edge, Edge::StraightEdge(_))
    }

    /// Checks if a given point is inside the puzzle piece
    /// Trick: Check if the point is inside the rotated subpath. If not, check if it is inside the original subpath
    fn contains(&self, point: DVec2) -> bool {
        self.subpath.point_inside(
            point,
            // self.rotation_matrix1,
            // self.rotation_matrix2,
            // &self.rotated_subpath1,
            // &self.rotated_subpath2,
        ) || self.subpath.contains_point(point)
    }

    #[allow(dead_code)]
    fn draw_debug_line(&self, image: &mut RgbaImage) {
        for path in self.subpath.iter() {
            match path.handles {
                BezierHandles::Linear => {
                    imageproc::drawing::draw_line_segment_mut(
                        image,
                        (path.start.x as f32, path.start.y as f32),
                        (path.end.x as f32, path.end.y as f32),
                        RED_COLOR,
                    );
                }
                BezierHandles::Quadratic { .. } => {}
                BezierHandles::Cubic {
                    handle_start,
                    handle_end,
                } => {
                    imageproc::drawing::draw_cubic_bezier_curve_mut(
                        image,
                        (path.start.x as f32, path.start.y as f32),
                        (path.end.x as f32, path.end.y as f32),
                        (handle_start.x as f32, handle_start.y as f32),
                        (handle_end.x as f32, handle_end.y as f32),
                        RED_COLOR,
                    );
                }
            }
        }
    }
}

const COMPARE_THRESHOLD: f32 = 10.0;

#[derive(Clone, PartialEq, Hash, Eq, Debug)]
pub struct PuzzleId(u64);

impl Identifier for PuzzleId {
    fn new() -> Self {
        PuzzleId(0)
    }
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum Side {
    Top,
    Right,
    Bottom,
    Left,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_divide_axis() {
        let res = divide_axis(1000.0, 4);
        assert_eq!(res.0.len(), 4);
        assert!(res.1 > 249.0 && res.1 < 251.0);
    }

    #[test]
    fn test_divisor_pairs() {
        let given_number = 1;
        assert_eq!(find_divisors(given_number), vec![(1, 1),]);

        let given_number = 24;
        assert_eq!(
            find_divisors(given_number),
            vec![
                (1, 24),
                (2, 12),
                (3, 8),
                (4, 6),
                (6, 4),
                (8, 3),
                (12, 2),
                (24, 1),
            ]
        );

        let given_number = 9;
        assert_eq!(find_divisors(given_number), vec![(1, 9), (3, 3), (9, 1),])
    }

    #[test]
    fn test_optimal_aspect_ratio() {
        let image_width: f32 = 1024.;
        let image_height: f32 = 768.;
        let possible_aspect_ratios = vec![(1, 25), (5, 5), (25, 1)];
        assert_eq!(
            optimal_aspect_ratio(possible_aspect_ratios, image_width, image_height),
            Ok((5, 5))
        );

        let image_width: f32 = 666.;
        let image_height: f32 = 666.;
        let possible_aspect_ratios = vec![
            (1, 24),
            (2, 12),
            (3, 8),
            (4, 6),
            (6, 4),
            (8, 3),
            (12, 2),
            (24, 1),
        ];
        assert_eq!(
            optimal_aspect_ratio(possible_aspect_ratios, image_width, image_height),
            Ok((6, 4))
        );
    }
}
