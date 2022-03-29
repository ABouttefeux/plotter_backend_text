#![doc = include_str!("../README.md")]
//
//#![warn(clippy::as_conversions)]
#![warn(clippy::cast_sign_loss)]
#![warn(clippy::cast_possible_truncation)]
#![warn(clippy::cast_possible_wrap)]
#![warn(clippy::clone_on_ref_ptr)]
#![warn(clippy::cognitive_complexity)]
#![warn(clippy::default_numeric_fallback)]
#![warn(clippy::float_cmp_const)]
#![warn(clippy::implicit_hasher)]
#![warn(clippy::implicit_saturating_sub)]
#![warn(clippy::imprecise_flops)]
#![warn(clippy::large_types_passed_by_value)]
#![warn(clippy::macro_use_imports)]
#![warn(clippy::manual_ok_or)]
#![warn(clippy::missing_const_for_fn)]
#![warn(clippy::needless_pass_by_value)]
#![warn(clippy::non_ascii_literal)]
#![warn(clippy::semicolon_if_nothing_returned)]
#![warn(clippy::suboptimal_flops)]
#![warn(clippy::todo)]
#![warn(clippy::trivially_copy_pass_by_ref)]
#![warn(clippy::type_repetition_in_bounds)]
#![warn(clippy::unreadable_literal)]
#![warn(clippy::unseparated_literal_suffix)]
#![warn(clippy::unused_self)]
#![warn(clippy::unnecessary_wraps)]
#![warn(clippy::missing_errors_doc)]
#![warn(missing_docs)]
#![forbid(unsafe_code)]
#![doc(html_root_url = "https://docs.rs/plotter_backend_text/0.1.0")]

#[cfg(test)]
mod test;

use std::convert::{TryFrom, TryInto};
use std::io::{self, Write};

use plotters::coord::ranged1d::AsRangedCoord;
use plotters::coord::types::RangedCoordf64;
use plotters::prelude::*;
use plotters::style::text_anchor::{HPos, VPos};
use plotters_backend::{
    BackendColor, BackendStyle, BackendTextStyle, DrawingBackend, DrawingErrorKind,
};
#[cfg(feature = "serde-serialize")]
use serde::{Deserialize, Serialize};

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde-serialize", derive(Serialize, Deserialize))]
/// State of the pixel in the canvas.
pub enum PixelState {
    /// The pixel is empty.
    Empty,
    /// The pixel is `-`
    HLine,
    /// The pixel is `|`
    VLine,
    /// The pixel is `+`
    Cross,
    /// The pixel is `x`
    Pixel,
    /// The pixel is a character.
    Text(char),
    /// the pixel a circle filled `@` or not `O`
    Circle(bool),
}

impl PixelState {
    /// Returns the character to draw.
    const fn to_char(self) -> char {
        match self {
            Self::Empty => ' ',
            Self::HLine => '-',
            Self::VLine => '|',
            Self::Cross => '+',
            Self::Pixel => '.',
            Self::Text(c) => c,
            Self::Circle(filled) => {
                if filled {
                    '@'
                } else {
                    'O'
                }
            }
        }
    }

    /// Updates the state of the pixel with a superposition of another state.
    fn update(&mut self, new_state: PixelState) {
        let next_state = match (*self, new_state) {
            (Self::HLine, Self::VLine) => Self::Cross,
            (Self::VLine, Self::HLine) => Self::Cross,
            (_, Self::Circle(what)) => Self::Circle(what),
            (Self::Circle(what), _) => Self::Circle(what),
            (_, Self::Pixel) => Self::Pixel,
            (Self::Pixel, _) => Self::Pixel,
            (_, new) => new,
        };

        *self = next_state;
    }
}

#[derive(Debug, Clone, Hash, Eq, PartialEq)]
#[cfg_attr(feature = "serde-serialize", derive(Serialize, Deserialize))]
/// Text Drawing Backend for the Plotters library.
pub struct TextDrawingBackend {
    /// Width of the canvas.
    pub size_x: u32,
    /// Height of the canvas.
    pub size_y: u32,
    /// Pixel of the canvas.
    pub pixels: Vec<PixelState>,
}

impl TextDrawingBackend {
    /// Creates a new `TextDrawingBackend` with the given size.
    pub fn new(size_x: u32, size_y: u32) -> Self {
        Self {
            size_x,
            size_y,
            pixels: vec![PixelState::Empty; (size_x * size_y).try_into().unwrap()],
        }
    }

    /// Getter on the pixels.
    pub fn pixels(&self) -> &[PixelState] {
        &self.pixels
    }

    /// Getter on the width of the canvas.
    pub const fn size_x(&self) -> u32 {
        self.size_x
    }

    /// Getter on the height of the canvas.
    pub const fn size_y(&self) -> u32 {
        self.size_y
    }

    /// Iterate over the pixels of the canvas.
    pub fn iter(&self) -> impl Iterator<Item = &PixelState> {
        self.pixels.iter()
    }

    /// Iterate over the pixels of the canvas.
    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut PixelState> {
        self.pixels.iter_mut()
    }

    /// Set the pixel at the given position to the given state.
    pub fn set_state(&mut self, pos_x: usize, pos_y: usize, p: PixelState) {
        let index: usize = pos_x + pos_y * usize::try_from(self.size_x).unwrap();
        if index < self.pixels.len() {
            self.pixels[index] = p;
        }
    }

    /// Update the pixel at the given position to the given state.
    pub fn update_state(&mut self, pos_x: usize, pos_y: usize, p: PixelState) {
        let index: usize = pos_x + pos_y * usize::try_from(self.size_x).unwrap();
        if index < self.pixels.len() {
            self.pixels[index].update(p);
        }
    }
}

impl Default for TextDrawingBackend {
    fn default() -> Self {
        Self::new(100, 30)
    }
}

impl<'a> IntoIterator for &'a TextDrawingBackend {
    type IntoIter = <&'a Vec<PixelState> as IntoIterator>::IntoIter;
    type Item = &'a PixelState;

    fn into_iter(self) -> Self::IntoIter {
        self.pixels.iter()
    }
}

impl<'a> IntoIterator for &'a mut TextDrawingBackend {
    type IntoIter = <&'a mut Vec<PixelState> as IntoIterator>::IntoIter;
    type Item = &'a mut PixelState;

    fn into_iter(self) -> Self::IntoIter {
        self.pixels.iter_mut()
    }
}

impl DrawingBackend for TextDrawingBackend {
    type ErrorType = std::io::Error;

    fn get_size(&self) -> (u32, u32) {
        (self.size_x, self.size_y)
    }

    fn ensure_prepared(&mut self) -> Result<(), DrawingErrorKind<std::io::Error>> {
        Ok(())
    }

    fn present(&mut self) -> Result<(), DrawingErrorKind<std::io::Error>> {
        let stderr = io::stderr();
        let mut handle = io::BufWriter::new(stderr);
        // we aquire the lock on stderr
        for r in 0..self.size_y {
            let mut buf = String::new();
            for c in 0..self.size_x {
                buf.push(self.pixels[(r * self.size_x + c) as usize].to_char());
            }
            writeln!(handle, "{}", buf).map_err(DrawingErrorKind::DrawingError)?;
        }

        Ok(())
    }

    #[allow(clippy::cast_sign_loss)]
    fn draw_pixel(
        &mut self,
        pos: (i32, i32),
        color: BackendColor,
    ) -> Result<(), DrawingErrorKind<std::io::Error>> {
        if color.alpha > 0.3_f64 {
            self.update_state(pos.0 as usize, pos.1 as usize, PixelState::Pixel);
        }
        Ok(())
    }

    #[allow(clippy::cast_possible_truncation)]
    #[allow(clippy::cast_sign_loss)]
    fn draw_line<S: BackendStyle>(
        &mut self,
        from: (i32, i32),
        to: (i32, i32),
        style: &S,
    ) -> Result<(), DrawingErrorKind<Self::ErrorType>> {
        if from.0 == to.0 {
            let x = from.0;
            let y0 = from.1.min(to.1);
            let y1 = from.1.max(to.1);
            for y in y0..y1 {
                self.pixels[(y * 100_i32 + x) as usize].update(PixelState::VLine);
            }
            return Ok(());
        }

        if from.1 == to.1 {
            let y = from.1;
            let x0 = from.0.min(to.0);
            let x1 = from.0.max(to.0);
            for x in x0..x1 {
                self.pixels[(y * 100_i32 + x) as usize].update(PixelState::HLine);
            }
            return Ok(());
        }

        plotters_backend::rasterizer::draw_line(self, from, to, style)
    }

    #[allow(clippy::cast_possible_truncation)]
    fn estimate_text_size<S: BackendTextStyle>(
        &self,
        text: &str,
        _: &S,
    ) -> Result<(u32, u32), DrawingErrorKind<Self::ErrorType>> {
        Ok((text.len() as u32, 1))
    }

    #[allow(clippy::cast_possible_wrap)]
    #[allow(clippy::cast_sign_loss)]
    fn draw_text<S: BackendTextStyle>(
        &mut self,
        text: &str,
        style: &S,
        pos: (i32, i32),
    ) -> Result<(), DrawingErrorKind<Self::ErrorType>> {
        let (width, height) = self.estimate_text_size(text, style)?;
        let (width, height) = (width as i32, height as i32);
        let dx = match style.anchor().h_pos {
            HPos::Left => 0_i32,
            HPos::Right => -width,
            HPos::Center => -width / 2_i32,
        };
        let dy = match style.anchor().v_pos {
            VPos::Top => 0_i32,
            VPos::Center => -height / 2_i32,
            VPos::Bottom => -height,
        };
        let offset = (pos.1 + dy).max(0_i32) * 100_i32 + (pos.0 + dx).max(0_i32);
        for (idx, chr) in (offset..).zip(text.chars()) {
            self.pixels[idx as usize].update(PixelState::Text(chr));
        }
        Ok(())
    }
}
/// Draw a chart on a given drawing area. With thw given range and series of data. `text` is the caption of the graph.
///
/// # Errors
/// Return an error if the drawing encounters an error.
pub fn draw_chart<DB: DrawingBackend>(
    b: &DrawingArea<DB, plotters::coord::Shift>,
    range_x: impl AsRangedCoord<CoordDescType = RangedCoordf64>,
    range_y: impl AsRangedCoord<CoordDescType = RangedCoordf64>,
    series: impl Iterator<Item = (f64, f64)>,
    text: &str,
) -> Result<(), DrawingAreaErrorKind<DB::ErrorType>> {
    let mut chart = ChartBuilder::on(b)
        .margin(1_i32)
        .caption(text, ("sans-serif", (10_i32).percent_height()))
        .set_label_area_size(LabelAreaPosition::Left, (5_i32).percent_width())
        .set_label_area_size(LabelAreaPosition::Bottom, (10_i32).percent_height())
        .build_cartesian_2d(range_x, range_y)?;

    chart
        .configure_mesh()
        .disable_x_mesh()
        .disable_y_mesh()
        .draw()?;

    chart.draw_series(LineSeries::new(series, &RED))?;

    b.present()?;

    Ok(())
}
