//! The Jigsaw Puzzle library creates SVG paths which can be used to cut out puzzle pieces from a
//! given rectangular image. It provides three public functions:
//!
//! - [`build_jigsaw_template`] returns the paths from a given number of pieces in a column and a
//! row. This is the function you normally want to use
//! - [`generate_columns_rows_numbers`] returns an ideal distribution of pieces on the x- and y-axes
//! for a given total number of pieces
//! - [`round`] is a util function which approximately rounds a f32 value to two decimal places

use image::{DynamicImage, GenericImageView};
use imageproc::drawing::{draw_cubic_bezier_curve_mut, draw_line_segment_mut};
use std::f32;
use std::vec;

const DEFAULT_TAB_SIZE: f32 = 20.0;
const DEFAULT_JITTER: f32 = 0.0;

/// Provides all information on how to cut out the jigsaw puzzle pieces from an image
#[derive(Clone, Debug)]
pub struct JigsawTemplate {
    /// SVG path for every jigsaw puzzle piece
    pub svg_paths: Vec<String>,
    /// The dimensions (width, length) in pixel
    pub piece_dimensions: (f32, f32),
    /// The number of pieces in the x- and the y-axis
    pub number_of_pieces: (usize, usize),
}

/// A segment of an indented puzzle piece edge. A segment is described by a cubic Bézier curve,
/// which includes a starting point, an end point and two control points. Three segments make up a
/// piece's edge.
#[derive(Clone, Debug)]
struct IndentationSegment {
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
    /// Return segment as SVG path. If reverse = true, return path from right to left for
    /// horizontal edges and bottom to top for vertical edges.
    pub fn as_path(&self, image: &mut DynamicImage, starting_point: (f32, f32), reverse: bool) -> String {
        draw_cubic_bezier_curve_mut(
            image,
            self.starting_point,
            self.end_point,
            self.control_point_1,
            self.control_point_2,
            image::Rgba([0, 0, 255, 255]),
        );
        if reverse {
            format!(
                "C {},{} {},{} {},{}",
                &self.control_point_2.0,
                &self.control_point_2.1,
                &self.control_point_1.0,
                &self.control_point_1.1,
                &self.starting_point.0,
                &self.starting_point.1
            )
        } else {
            format!(
                "C {},{} {},{} {},{}",
                &self.control_point_1.0,
                &self.control_point_1.1,
                &self.control_point_2.0,
                &self.control_point_2.1,
                &self.end_point.0,
                &self.end_point.1
            )
        }
    }
}

#[derive(Clone, Debug)]
/// An indented puzzle piece edge. An edge is decribe via three distinct cubic Bézier curves (the
/// "segments")
struct IndentedEdge {
    /// Describes the left half for a horizontal edge, the upper half for a vertical edge
    pub first_segment: IndentationSegment,
    /// Describes the form of the tab
    pub middle_segment: IndentationSegment,
    /// Describes the right half for a horizontal edge, the lower half for a vertical edge
    pub last_segment: IndentationSegment,
}

impl IndentedEdge {
    /// Creates a new indented edge
    pub fn new(
        starting_point: (f32, f32),
        end_point: (f32, f32),
        generator: &mut EdgeContourGenerator,
    ) -> Self {
        generator.create(starting_point, end_point)
    }
    /// Returns edge as SVG path. If reverse = true, returns path from right to left for horizontal
    /// edges and bottom to top for vertical edges
    pub fn as_path(&self, image: &mut DynamicImage, starting_point: (f32, f32), reverse: bool) -> String {
        if reverse {
            format!(
                "{}  {}  {}",
                &self.last_segment.as_path(image, starting_point, reverse),
                &self.middle_segment.as_path(image, starting_point, reverse),
                &self.first_segment.as_path(image, starting_point, reverse)
            )
        } else {
            format!(
                "{}  {}  {}",
                &self.first_segment.as_path(image, starting_point, reverse),
                &self.middle_segment.as_path(image, starting_point, reverse),
                &self.last_segment.as_path(image, starting_point, reverse)
            )
        }
    }
}

/// Provides the means to generate [`IndentedEdge`]s
struct EdgeContourGenerator {
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
    /// length and the direction of the edge (horizontal, vertical) and finally two coefficients
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

#[derive(Clone, Debug)]
/// A puzzle piece edge which is at the same time a part of the puzzle's border and therefore forms
/// a straight line
struct StraightEdge {
    pub starting_point: (f32, f32),
    pub end_point: (f32, f32),
}

impl StraightEdge {
    /// Returns edge as SVG path. If reverse = true, returns path from right to left for horizontal
    /// edges and bottom to top for vertical edges
    pub fn as_path(&self, image: &mut DynamicImage, starting_point: (f32, f32), reverse: bool) -> String {
        draw_line_segment_mut(
            image,
            self.starting_point,
            self.end_point,
            image::Rgba([255, 0, 0, 255]),
        );
        if reverse {
            format!("L {},{}", self.starting_point.0, self.starting_point.1)
        } else {
            format!("L {},{}", self.end_point.0, self.end_point.1)
        }
    }
}

#[derive(Clone, Debug)]
/// A border of a puzzle piece. Can be either an `StraightEdge` (no adjacent other piece) or an
/// `IndentedEdge`
enum Edge {
    IndentedEdge(IndentedEdge),
    StraightEdge(StraightEdge),
}

impl Edge {
    /// Returns edge as SVG path. If reverse = true, returns path from right to left for horizontal
    /// edges and bottom to top for vertical edges
    pub fn as_path(&self, image: &mut DynamicImage, starting_point: (f32, f32), reverse: bool) -> String {
        match self {
            Edge::IndentedEdge(ie) => ie.as_path(image, starting_point, reverse),
            Edge::StraightEdge(oe) => oe.as_path(image, starting_point, reverse),
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

/// Takes the for edges of a "piece" and creates an SVG path around it. The path always starts in
/// the upper left corner and proceeds clockwise.
fn puzzle_piece(
    image: &mut DynamicImage,
    starting_point: (f32, f32),
    top_edge: &Edge,
    right_edge: &Edge,
    bottom_edge: &Edge,
    left_edge: &Edge,
) -> String {
    format!(
        "M {},{}  {}  {}  {}  {}  Z",
        starting_point.0,
        starting_point.1,
        top_edge.as_path(image, starting_point, false),
        right_edge.as_path(image, starting_point, false),
        bottom_edge.as_path(image, starting_point, true),
        left_edge.as_path(image, starting_point, true)
    )
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
fn end_point_pos(ind: usize, segments: &Vec<f32>, fallback: f32) -> f32 {
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
) -> (usize, usize) {
    let mut width_height_diff = std::f32::MAX;
    let mut number_of_pieces = *possible_dimensions
        .first()
        .expect("No possible dimensions found. This error should never happen!");
    for (x, y) in possible_dimensions {
        let width = image_width / x as f32;
        let height = image_height / y as f32;
        let new_width_height_diff = (width - height).abs();
        if new_width_height_diff < 1. {
            return (x, y);
        }
        if width_height_diff >= new_width_height_diff {
            width_height_diff = new_width_height_diff;
            number_of_pieces = (x, y);
        } else {
            return number_of_pieces;
        }
    }
    number_of_pieces
}

/// Returns the visually most appealing numbers of pieces in one column and one row based on a
/// given number of pieces
pub fn generate_columns_rows_numbers(
    image_width: f32,
    image_height: f32,
    number_of_pieces: usize,
) -> (usize, usize) {
    let divisor_pairs = find_divisors(number_of_pieces);
    optimal_aspect_ratio(divisor_pairs, image_width, image_height)
}

/// Returns information on how to cut jigsaw puzzle pieces from an image of a given width and
/// height and the number of pieces in a row and a column as an optional the tab size, a "jitter"
/// factor and a initial seed value.
///
/// The `tab_size` argument defines the size of the pieces' tabs. It can be any number from `10.0` to `30.0` and defaults to `20.0`
///
/// `jitter` can be a number between 0.0 and 13.0. The bigger the number, the more asymmetrical are
/// the puzzle pieces. Defaults to `0.0` (symmetrical).
///
/// `seed` provides the initial "randomness" when creating the contours of the puzzle pieces. Same
/// seed values for images with same dimensions and same number of pieces lead to same SVG paths.
pub fn build_jigsaw_template(
    image: DynamicImage,
    pieces_in_column: usize,
    pieces_in_row: usize,
    tab_size: Option<f32>,
    jitter: Option<f32>,
    seed: Option<usize>,
) -> DynamicImage {
    let (image_width, image_height) = image.dimensions();
    let image_width = image_width as f32;
    let image_height = image_height as f32;
    let (starting_points_x, piece_width) = divide_axis(image_width, pieces_in_column);
    let (starting_points_y, piece_height) = divide_axis(image_height, pieces_in_row);
    let mut contour_gen =
        EdgeContourGenerator::new(piece_width, piece_height, tab_size, jitter, seed);
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
    let mut i = 0;
    let mut image = image.clone();
    for y in starting_points_y.iter() {
        for x in starting_points_x.iter() {
            let (top_index, right_index, bottom_index, left_index) =
                get_border_indices(i, pieces_in_column);


            // draw_cubic_bezier_curve_mut(&mut image, (*x, *y), (*x, *y + piece_height), (*x + piece_width, *y + piece_height), (*x + piece_width, *y), image::Rgba([0, 0, 0, 255]));
            puzzle_piece(
                &mut image,
                (*x, *y),
                &horizontal_edges[top_index],
                &vertical_edges[right_index],
                &horizontal_edges[bottom_index],
                &vertical_edges[left_index],
            );
            let mut tile = image.view(*x as u32, *y as u32, piece_width as u32, piece_height as u32).to_image();
            tile.save(format!("tiles/puzzle_piece_{}.png", i)).expect("Failed to save piece");

            i += 1;
        }
    };
    // JigsawTemplate {
    //     svg_paths,
    //     piece_dimensions: (piece_width, piece_height),
    //     number_of_pieces: (pieces_in_column, pieces_in_row),
    // }
    image
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_divide_axis() {
        let res = divide_axis(1000.0, 4);
        assert!(res.0.len() == 4);
        assert!(res.1 > 249.0 && res.1 < 251.0);
    }

    #[test]
    fn test_divisor_pairs() {
        let given_number = 1;
        assert_eq!(find_divisors(given_number), vec![(1, 1), ]);

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
        assert_eq!(find_divisors(given_number), vec![(1, 9), (3, 3), (9, 1), ])
    }

    #[test]
    fn test_optimal_aspect_ratio() {
        let image_width: f32 = 1024.;
        let image_height: f32 = 768.;
        let possible_aspect_ratios = vec![(1, 25), (5, 5), (25, 1)];
        assert_eq!(
            optimal_aspect_ratio(possible_aspect_ratios, image_width, image_height),
            (5, 5)
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
            (6, 4)
        );
    }
}
