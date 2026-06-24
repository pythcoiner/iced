//! Draw lines around containers.
use crate::{Color, Pixels, Size};

/// A border.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Border {
    /// The color of the border.
    pub color: Color,

    /// The width of the border.
    pub width: f32,

    /// The [`Radius`] of the border.
    pub radius: Radius,

    /// The line [`Style`] of the border.
    pub style: Style,
}

/// Creates a new [`Border`] with the given [`Radius`].
///
/// ```
/// # use iced_core::border::{self, Border};
/// #
/// assert_eq!(border::rounded(10), Border::default().rounded(10));
/// ```
pub fn rounded(radius: impl Into<Radius>) -> Border {
    Border::default().rounded(radius)
}

/// Creates a new [`Border`] with the given [`Color`].
///
/// ```
/// # use iced_core::border::{self, Border};
/// # use iced_core::Color;
/// #
/// assert_eq!(border::color(Color::BLACK), Border::default().color(Color::BLACK));
/// ```
pub fn color(color: impl Into<Color>) -> Border {
    Border::default().color(color)
}

/// Creates a new [`Border`] with the given `width`.
///
/// ```
/// # use iced_core::border::{self, Border};
/// # use iced_core::Color;
/// #
/// assert_eq!(border::width(10), Border::default().width(10));
/// ```
pub fn width(width: impl Into<Pixels>) -> Border {
    Border::default().width(width)
}

impl Border {
    /// Sets the [`Color`] of the [`Border`].
    pub fn color(self, color: impl Into<Color>) -> Self {
        Self {
            color: color.into(),
            ..self
        }
    }

    /// Sets the [`Radius`] of the [`Border`].
    pub fn rounded(self, radius: impl Into<Radius>) -> Self {
        Self {
            radius: radius.into(),
            ..self
        }
    }

    /// Sets the width of the [`Border`].
    pub fn width(self, width: impl Into<Pixels>) -> Self {
        Self {
            width: width.into().0,
            ..self
        }
    }

    /// Sets the line [`Style`] of the [`Border`].
    pub fn style(self, style: Style) -> Self {
        Self { style, ..self }
    }

    /// Draws the [`Border`] with the given dashed [`Dash`] pattern.
    pub fn dashes(self, dash: Dash) -> Self {
        self.style(Style::Dashed(dash))
    }

    /// Draws the [`Border`] with the given dotted [`Dash`] pattern.
    pub fn dots(self, dash: Dash) -> Self {
        self.style(Style::Dotted(dash))
    }

    /// Draws the [`Border`] as dashes with short, dense spacing.
    pub fn dashed_tight(self) -> Self {
        self.dashes(Dash {
            length: self.width * 2.0,
            gap: self.width * 2.0,
        })
    }

    /// Draws the [`Border`] as dashes with even spacing.
    pub fn dashed(self) -> Self {
        self.dashes(Dash {
            length: self.width * 4.0,
            gap: self.width * 3.0,
        })
    }

    /// Draws the [`Border`] as dashes with long lines and wide gaps.
    pub fn dashed_loose(self) -> Self {
        self.dashes(Dash {
            length: self.width * 7.0,
            gap: self.width * 4.0,
        })
    }

    /// Draws the [`Border`] as dots with short, dense spacing.
    pub fn dotted_tight(self) -> Self {
        self.dots(Dash {
            length: 0.0,
            gap: self.width * 2.0,
        })
    }

    /// Draws the [`Border`] as dots with even spacing.
    pub fn dotted(self) -> Self {
        self.dots(Dash {
            length: 0.0,
            gap: self.width * 3.0,
        })
    }

    /// Draws the [`Border`] as dots with wide spacing.
    pub fn dotted_loose(self) -> Self {
        self.dots(Dash {
            length: 0.0,
            gap: self.width * 4.0,
        })
    }
}

/// The line style of a [`Border`].
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum Style {
    /// A continuous solid line.
    #[default]
    Solid,
    /// A series of square dashes separated by gaps.
    Dashed(Dash),
    /// A series of round dots separated by gaps.
    Dotted(Dash),
}

/// A dash pattern for a [`Border`], in logical pixels.
///
/// For [`Style::Dotted`] borders, the dots are as wide as the border and a
/// `length` of `0.0` draws round dots; only the `gap` controls their spacing.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Dash {
    /// The length of each drawn segment.
    pub length: f32,

    /// The length of each gap between segments.
    pub gap: f32,
}

impl Dash {
    /// Creates a new [`Dash`] with the given segment and gap lengths.
    pub fn new(length: impl Into<Pixels>, gap: impl Into<Pixels>) -> Self {
        Self {
            length: length.into().0,
            gap: gap.into().0,
        }
    }
}

/// Computes the perimeter of a rounded rectangle with the given `size` and
/// corner `radius`.
pub fn perimeter(size: Size, radius: Radius) -> f32 {
    let radius_limit = size.width.min(size.height) / 2.0;
    let radii = <[f32; 4]>::from(radius).map(|radius| radius.min(radius_limit));
    let radius_sum: f32 = radii.into_iter().sum();

    2.0 * (size.width + size.height)
        - (2.0 - std::f32::consts::FRAC_PI_2) * radius_sum
}

/// A border [`Style`] resolved to concrete dash segments for a given width.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Segments {
    /// The length of each drawn segment.
    pub on: f32,

    /// The length of each gap between segments.
    pub off: f32,

    /// Whether the segments are drawn with round caps, as for dotted borders.
    pub rounded: bool,
}

impl Segments {
    /// Scales the pattern so it repeats an integer number of times along a
    /// segment of the given `length`.
    pub fn fit_to_length(self, length: f32) -> Self {
        let period = self.on + self.off;

        if period <= 0.0 || length <= 0.0 {
            return self;
        }

        let count = (length / period).round().max(1.0);
        let scale = length / (count * period);

        Self {
            on: self.on * scale,
            off: self.off * scale,
            ..self
        }
    }

    /// Scales the pattern so it repeats an integer number of times along the
    /// given `perimeter`.
    pub fn fit_to_perimeter(self, perimeter: f32) -> Self {
        self.fit_to_length(perimeter)
    }
}

/// The smallest segment length, in logical pixels, that keeps stroking stable.
const MIN_SEGMENT: f32 = 0.1;

impl Style {
    /// Resolves the [`Style`] into dash [`Segments`] for a border of the given
    /// `width`, or `None` for a [`Style::Solid`] border.
    pub fn segments(self, width: f32) -> Option<Segments> {
        match self {
            Style::Solid => None,
            Style::Dashed(dash) => Some(Segments {
                on: dash.length.max(MIN_SEGMENT),
                off: dash.gap.max(MIN_SEGMENT),
                rounded: false,
            }),
            Style::Dotted(dash) => Some(Segments {
                on: dash.length.max(MIN_SEGMENT),
                // Round caps spread each dot by `width`, so widen the gap to keep
                // the spacing between dots equal to `gap`.
                off: (dash.gap + width).max(MIN_SEGMENT),
                rounded: true,
            }),
        }
    }
}

/// The border radii for the corners of a graphics primitive in the order:
/// top-left, top-right, bottom-right, bottom-left.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Radius {
    /// Top left radius
    pub top_left: f32,
    /// Top right radius
    pub top_right: f32,
    /// Bottom right radius
    pub bottom_right: f32,
    /// Bottom left radius
    pub bottom_left: f32,
}

/// Creates a new [`Radius`] with the same value for each corner.
pub fn radius(value: impl Into<Pixels>) -> Radius {
    Radius::new(value)
}

/// Creates a new [`Radius`] with the given top left value.
pub fn top_left(value: impl Into<Pixels>) -> Radius {
    Radius::default().top_left(value)
}

/// Creates a new [`Radius`] with the given top right value.
pub fn top_right(value: impl Into<Pixels>) -> Radius {
    Radius::default().top_right(value)
}

/// Creates a new [`Radius`] with the given bottom right value.
pub fn bottom_right(value: impl Into<Pixels>) -> Radius {
    Radius::default().bottom_right(value)
}

/// Creates a new [`Radius`] with the given bottom left value.
pub fn bottom_left(value: impl Into<Pixels>) -> Radius {
    Radius::default().bottom_left(value)
}

/// Creates a new [`Radius`] with the given value as top left and top right.
pub fn top(value: impl Into<Pixels>) -> Radius {
    Radius::default().top(value)
}

/// Creates a new [`Radius`] with the given value as bottom left and bottom right.
pub fn bottom(value: impl Into<Pixels>) -> Radius {
    Radius::default().bottom(value)
}

/// Creates a new [`Radius`] with the given value as top left and bottom left.
pub fn left(value: impl Into<Pixels>) -> Radius {
    Radius::default().left(value)
}

/// Creates a new [`Radius`] with the given value as top right and bottom right.
pub fn right(value: impl Into<Pixels>) -> Radius {
    Radius::default().right(value)
}

impl Radius {
    /// Creates a new [`Radius`] with the same value for each corner.
    pub fn new(value: impl Into<Pixels>) -> Self {
        let value = value.into().0;

        Self {
            top_left: value,
            top_right: value,
            bottom_right: value,
            bottom_left: value,
        }
    }

    /// Sets the top left value of the [`Radius`].
    pub fn top_left(self, value: impl Into<Pixels>) -> Self {
        Self {
            top_left: value.into().0,
            ..self
        }
    }

    /// Sets the top right value of the [`Radius`].
    pub fn top_right(self, value: impl Into<Pixels>) -> Self {
        Self {
            top_right: value.into().0,
            ..self
        }
    }

    /// Sets the bottom right value of the [`Radius`].
    pub fn bottom_right(self, value: impl Into<Pixels>) -> Self {
        Self {
            bottom_right: value.into().0,
            ..self
        }
    }

    /// Sets the bottom left value of the [`Radius`].
    pub fn bottom_left(self, value: impl Into<Pixels>) -> Self {
        Self {
            bottom_left: value.into().0,
            ..self
        }
    }

    /// Sets the top left and top right values of the [`Radius`].
    pub fn top(self, value: impl Into<Pixels>) -> Self {
        let value = value.into().0;

        Self {
            top_left: value,
            top_right: value,
            ..self
        }
    }

    /// Sets the bottom left and bottom right values of the [`Radius`].
    pub fn bottom(self, value: impl Into<Pixels>) -> Self {
        let value = value.into().0;

        Self {
            bottom_left: value,
            bottom_right: value,
            ..self
        }
    }

    /// Sets the top left and bottom left values of the [`Radius`].
    pub fn left(self, value: impl Into<Pixels>) -> Self {
        let value = value.into().0;

        Self {
            top_left: value,
            bottom_left: value,
            ..self
        }
    }

    /// Sets the top right and bottom right values of the [`Radius`].
    pub fn right(self, value: impl Into<Pixels>) -> Self {
        let value = value.into().0;

        Self {
            top_right: value,
            bottom_right: value,
            ..self
        }
    }
}

impl From<f32> for Radius {
    fn from(radius: f32) -> Self {
        Self {
            top_left: radius,
            top_right: radius,
            bottom_right: radius,
            bottom_left: radius,
        }
    }
}

impl From<u8> for Radius {
    fn from(w: u8) -> Self {
        Self::from(f32::from(w))
    }
}

impl From<u32> for Radius {
    fn from(w: u32) -> Self {
        Self::from(w as f32)
    }
}

impl From<i32> for Radius {
    fn from(w: i32) -> Self {
        Self::from(w as f32)
    }
}

impl From<Radius> for [f32; 4] {
    fn from(radi: Radius) -> Self {
        [
            radi.top_left,
            radi.top_right,
            radi.bottom_right,
            radi.bottom_left,
        ]
    }
}

impl std::ops::Mul<f32> for Radius {
    type Output = Self;

    fn mul(self, scale: f32) -> Self::Output {
        Self {
            top_left: self.top_left * scale,
            top_right: self.top_right * scale,
            bottom_right: self.bottom_right * scale,
            bottom_left: self.bottom_left * scale,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn solid_has_no_segments() {
        assert_eq!(Style::Solid.segments(2.0), None);
    }

    #[test]
    fn dashed_segments_are_length_and_gap() {
        let style = Style::Dashed(Dash {
            length: 6.0,
            gap: 4.0,
        });

        assert_eq!(
            style.segments(2.0),
            Some(Segments {
                on: 6.0,
                off: 4.0,
                rounded: false,
            })
        );
    }

    #[test]
    fn dotted_rounds_and_widens_the_gap_by_the_width() {
        let style = Style::Dotted(Dash {
            length: 0.0,
            gap: 4.0,
        });

        assert_eq!(
            style.segments(2.0),
            Some(Segments {
                on: MIN_SEGMENT,
                off: 6.0,
                rounded: true,
            })
        );
    }

    #[test]
    fn perimeter_clamps_radii_to_half_the_shortest_side() {
        let size = Size::new(20.0, 10.0);

        assert_eq!(
            perimeter(size, Radius::new(10.0)),
            20.0 + 10.0 * std::f32::consts::PI
        );
    }

    #[test]
    fn fit_to_perimeter_leaves_matching_period_unchanged() {
        let segments = Segments {
            on: 6.0,
            off: 4.0,
            rounded: false,
        };

        assert_eq!(segments.fit_to_perimeter(40.0), segments);
    }

    #[test]
    fn fit_to_perimeter_scales_to_integer_period_count() {
        let segments = Segments {
            on: 6.0,
            off: 4.0,
            rounded: false,
        }
        .fit_to_perimeter(42.0);

        assert!((segments.on - 6.3).abs() < 0.000_001);
        assert!((segments.off - 4.2).abs() < 0.000_001);
    }

    #[test]
    fn fit_to_perimeter_leaves_degenerate_pattern_unchanged() {
        let segments = Segments {
            on: 0.0,
            off: 0.0,
            rounded: true,
        };

        assert_eq!(segments.fit_to_perimeter(42.0), segments);
        assert_eq!(segments.fit_to_perimeter(0.0), segments);
    }
}
