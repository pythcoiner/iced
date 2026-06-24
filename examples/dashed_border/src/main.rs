//! This example showcases the dashed and dotted border styles.
use iced::widget::{column, container, row, scrollable, text};
use iced::{Border, Center, Color, Element, Fill};

pub fn main() -> iced::Result {
    iced::run(update, view)
}

#[derive(Default)]
struct State;

#[derive(Debug, Clone)]
enum Message {}

fn update(_state: &mut State, _message: Message) {}

fn view(_state: &State) -> Element<'_, Message> {
    let base = Border::default()
        .width(3.0)
        .color(Color::from_rgb(0.31, 0.55, 0.95));

    let samples = column![
        group("Solid", base),
        group("Dashed", base.dashed()),
        group("Dashed (tight)", base.dashed_tight()),
        group("Dashed (loose)", base.dashed_loose()),
        group("Dotted", base.dotted()),
        group("Dotted (tight)", base.dotted_tight()),
        group("Dotted (loose)", base.dotted_loose()),
        group("Custom", base.dashes(iced::border::Dash::new(14.0, 6.0))),
    ]
    .spacing(24);

    scrollable(container(samples).padding(40).center_x(Fill)).into()
}

/// A labeled row showing the same border style on different corner shapes.
fn group<'a>(label: &'a str, border: Border) -> Element<'a, Message> {
    row![
        text(label).width(140),
        boxed(border),
        boxed(border.rounded(16.0)),
        boxed(
            border.rounded(
                iced::border::Radius::default()
                    .top_left(16.0)
                    .bottom_right(16.0),
            )
        ),
    ]
    .spacing(24)
    .align_y(Center)
    .into()
}

fn boxed<'a>(border: Border) -> Element<'a, Message> {
    container(text(""))
        .width(160)
        .height(70)
        .style(move |_theme| container::Style {
            border,
            ..container::Style::default()
        })
        .into()
}
