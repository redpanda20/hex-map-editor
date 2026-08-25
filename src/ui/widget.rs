#! Colour picker widget

use iced::{
    Color, Element, Length, Point, Renderer, Theme, mouse,
    widget::{
        Action,
        canvas::{self, Frame, Path, Stroke},
        column, responsive, row,
    },
};

use crate::{app::Message, domain::colour::Hsva};

pub fn colour_picker<'a>(
    colour: Color,
    on_change: impl Fn(Color) -> Message + 'a + Copy,
    on_commit: impl Fn(Color) -> Message + 'a + Copy,
) -> Element<'a, Message> {
    responsive(move |size| {
        const PICKER_THICKNESS: f32 = 20.0;
        const GAP: f32 = 8.0;
        const ROW_OVERHEAD: f32 = PICKER_THICKNESS * 2.0 + GAP * 2.0;

        let width = size.width;
        let height = size.height;

        // The square is allowed to shrink up to 100 px to accomidate sliders
        let is_col_layout = width < ROW_OVERHEAD * 2.0 + GAP * 2.0 + 100.0;

        // Calculate short edge length
        let side = match is_col_layout {
            true => width.clamp(100.0, 300.0),
            false => (width - ROW_OVERHEAD).min(height).clamp(100.0, 300.0),
        };

        let Hsva {
            hue,
            saturation,
            value,
            alpha,
        } = Hsva::from(colour);

        let sv_picker = SaturationValuePicker {
            alpha,
            hue,
            on_change: Box::new(on_change),
            on_commit: Box::new(on_commit),
        };

        let hue_picker = HuePicker {
            alpha,
            saturation,
            value,
            on_change: Box::new(on_change),
            on_commit: Box::new(on_commit),
            is_row: is_col_layout,
        };

        let alpha_picker = AlphaPicker {
            saturation,
            value,
            hue,
            on_change: Box::new(on_change),
            on_commit: Box::new(on_commit),
            is_row: is_col_layout,
        };

        if is_col_layout {
            column![
                iced::widget::canvas(PickerProgram(sv_picker))
                    .width(Length::Fixed(side))
                    .height(Length::Fixed(side)),
                iced::widget::canvas(PickerProgram(hue_picker))
                    .width(Length::Fixed(side))
                    .height(Length::Fixed(PICKER_THICKNESS)),
                iced::widget::canvas(PickerProgram(alpha_picker))
                    .width(Length::Fixed(side))
                    .height(Length::Fixed(PICKER_THICKNESS)),
            ]
            .spacing(GAP)
            .into()
        } else {
            row![
                iced::widget::canvas(PickerProgram(sv_picker))
                    .width(Length::Fixed(side))
                    .height(Length::Fixed(side)),
                iced::widget::canvas(PickerProgram(hue_picker))
                    .width(Length::Fixed(PICKER_THICKNESS))
                    .height(Length::Fixed(side)),
                iced::widget::canvas(PickerProgram(alpha_picker))
                    .width(Length::Fixed(PICKER_THICKNESS))
                    .height(Length::Fixed(side)),
            ]
            .spacing(GAP)
            .into()
        }
    })
    .into()
}

#[derive(Debug, Default)]
pub struct PickerState {
    is_selecting: bool,
    selected_point: Point,
}

trait Picker {
    fn on_change(&self) -> &dyn Fn(Color) -> Message;
    fn on_commit(&self) -> &dyn Fn(Color) -> Message;

    fn relative_position_to_colour(&self, relative_pos: Point, bounds: iced::Rectangle) -> Color;
    fn draw(&self, state: &PickerState, frame: &mut canvas::Frame, bounds: iced::Rectangle);
}

/// Saturation value square
pub struct SaturationValuePicker<'a> {
    alpha: f32,
    hue: f32,
    on_change: Box<dyn Fn(Color) -> Message + 'a>,
    on_commit: Box<dyn Fn(Color) -> Message + 'a>,
}

impl<'a> Picker for SaturationValuePicker<'a> {
    fn on_change(&self) -> &dyn Fn(Color) -> Message {
        &self.on_change
    }
    fn on_commit(&self) -> &dyn Fn(Color) -> Message {
        &self.on_commit
    }

    fn relative_position_to_colour(&self, relative_pos: Point, bounds: iced::Rectangle) -> Color {
        let saturation = relative_pos.x / bounds.width;
        let value = 1.0 - relative_pos.y / bounds.height;

        Hsva {
            hue: self.hue,
            saturation,
            value,
            alpha: self.alpha,
        }
        .into()
    }

    fn draw(&self, state: &PickerState, frame: &mut canvas::Frame, bounds: iced::Rectangle) {
        let origin = Point { x: 0.0, y: 0.0 };
        let right = Point {
            x: bounds.width,
            y: 0.0,
        };
        let bottom = Point {
            x: 0.0,
            y: bounds.height,
        };

        // Saturation gradient : Left to right
        let sat_gradient = canvas::gradient::Linear::new(origin, right)
            .add_stop(0.0, Color::WHITE)
            .add_stop(
                1.0,
                Hsva {
                    hue: self.hue,
                    saturation: 1.0,
                    value: 1.0,
                    alpha: 1.0,
                }
                .into(),
            );

        frame.fill_rectangle(Point::ORIGIN, bounds.size(), sat_gradient);

        // Value gradient : Top to bottom
        let val_gradient = canvas::gradient::Linear::new(origin, bottom)
            .add_stop(0.0, Color::TRANSPARENT)
            .add_stop(1.0, Color::BLACK);
        frame.fill_rectangle(Point::ORIGIN, bounds.size(), val_gradient);

        let sat_indicator = Path::line(
            Point {
                x: state.selected_point.x,
                y: 0.0,
            },
            Point {
                x: state.selected_point.x,
                y: bounds.height,
            },
        );
        frame.stroke(&sat_indicator, Stroke::default());

        let val_indicator = Path::line(
            Point {
                x: 0.0,
                y: state.selected_point.y,
            },
            Point {
                x: bounds.width,
                y: state.selected_point.y,
            },
        );
        frame.stroke(&val_indicator, Stroke::default());
    }
}

/// Hue colour line
pub struct HuePicker<'a> {
    saturation: f32,
    value: f32,
    alpha: f32,
    on_change: Box<dyn Fn(Color) -> Message + 'a>,
    on_commit: Box<dyn Fn(Color) -> Message + 'a>,
    is_row: bool,
}

impl<'a> Picker for HuePicker<'a> {
    fn on_change(&self) -> &dyn Fn(Color) -> Message {
        &self.on_change
    }
    fn on_commit(&self) -> &dyn Fn(Color) -> Message {
        &self.on_commit
    }

    fn relative_position_to_colour(&self, relative_pos: Point, bounds: iced::Rectangle) -> Color {
        let hue = match self.is_row {
            true => (relative_pos.x / bounds.width) * 360.0,
            false => (relative_pos.y / bounds.height) * 360.0,
        };

        Hsva {
            hue,
            saturation: self.saturation,
            value: self.value,
            alpha: self.alpha,
        }
        .into()
    }

    fn draw(&self, state: &PickerState, frame: &mut canvas::Frame, bounds: iced::Rectangle) {
        let origin = Point { x: 0.0, y: 0.0 };
        let end = match self.is_row {
            false => Point {
                x: 0.0,
                y: bounds.height,
            },
            true => Point {
                x: bounds.width,
                y: 0.0,
            },
        };

        let hue_gradient = canvas::gradient::Linear::new(origin, end)
            .add_stop(0.00, Color::from_rgba(1.0, 0.0, 0.0, 1.0))
            .add_stop(0.16, Color::from_rgba(1.0, 1.0, 0.0, 1.0))
            .add_stop(0.33, Color::from_rgba(0.0, 1.0, 0.0, 1.0))
            .add_stop(0.50, Color::from_rgba(0.0, 1.0, 1.0, 1.0))
            .add_stop(0.66, Color::from_rgba(0.0, 0.0, 1.0, 1.0))
            .add_stop(0.66, Color::from_rgba(0.0, 0.0, 1.0, 1.0))
            .add_stop(0.83, Color::from_rgba(1.0, 0.0, 1.0, 1.0))
            .add_stop(1.00, Color::from_rgba(1.0, 0.0, 0.0, 1.0));

        frame.fill_rectangle(Point::ORIGIN, bounds.size(), hue_gradient);

        let hue_indicator = match self.is_row {
            false => Path::line(
                Point {
                    x: 0.0,
                    y: state.selected_point.y,
                },
                Point {
                    x: bounds.width,
                    y: state.selected_point.y,
                },
            ),
            true => Path::line(
                Point {
                    x: state.selected_point.x,
                    y: 0.0,
                },
                Point {
                    x: state.selected_point.x,
                    y: bounds.height,
                },
            ),
        };
        frame.stroke(&hue_indicator, Stroke::default());
    }
}

/// Alpha value selector
pub struct AlphaPicker<'a> {
    saturation: f32,
    value: f32,
    hue: f32,
    on_change: Box<dyn Fn(Color) -> Message + 'a>,
    on_commit: Box<dyn Fn(Color) -> Message + 'a>,
    is_row: bool,
}

impl<'a> Picker for AlphaPicker<'a> {
    fn on_change(&self) -> &dyn Fn(Color) -> Message {
        &self.on_change
    }
    fn on_commit(&self) -> &dyn Fn(Color) -> Message {
        &self.on_commit
    }

    fn relative_position_to_colour(&self, relative_pos: Point, bounds: iced::Rectangle) -> Color {
        let alpha = match self.is_row {
            true => relative_pos.x / bounds.width,
            false => relative_pos.y / bounds.height,
        };

        Hsva {
            hue: self.hue,
            saturation: self.saturation,
            value: self.value,
            alpha,
        }
        .into()
    }

    fn draw(&self, state: &PickerState, frame: &mut canvas::Frame, bounds: iced::Rectangle) {
        // Checkerboard background
        let checkerboard_size = match self.is_row {
            false => bounds.width,
            true => bounds.height,
        } / 4.0;

        let cols = (bounds.width / checkerboard_size).ceil() as usize;
        let rows = (bounds.height / checkerboard_size).ceil() as usize;

        for row in 0..rows {
            for col in 0..cols {
                let colour = match (row + col) % 2 == 0 {
                    true => Color::from_rgb(0.0, 0.0, 0.0),
                    false => Color::from_rgb(0.3, 0.3, 0.3),
                };

                let tile_origin = Point {
                    x: col as f32 * checkerboard_size,
                    y: row as f32 * checkerboard_size,
                };
                let tile_size = iced::Size::new(checkerboard_size, checkerboard_size);

                frame.fill_rectangle(tile_origin, tile_size, colour);
            }
        }

        let origin = Point { x: 0.0, y: 0.0 };
        let end = match self.is_row {
            false => Point {
                x: 0.0,
                y: bounds.height,
            },
            true => Point {
                x: bounds.width,
                y: 0.0,
            },
        };

        // Transparent to Opaque : Left <--> Right
        let alpha_gradient = canvas::gradient::Linear::new(origin, end)
            .add_stop(
                0.00,
                Hsva {
                    hue: self.hue,
                    saturation: self.saturation,
                    value: self.value,
                    alpha: 0.0,
                }
                .into(),
            )
            .add_stop(
                1.00,
                Hsva {
                    hue: self.hue,
                    saturation: self.saturation,
                    value: self.value,
                    alpha: 1.0,
                }
                .into(),
            );

        frame.fill_rectangle(Point::ORIGIN, bounds.size(), alpha_gradient);

        let alpha_indicator = match self.is_row {
            false => Path::line(
                Point {
                    x: 0.0,
                    y: state.selected_point.y,
                },
                Point {
                    x: bounds.width,
                    y: state.selected_point.y,
                },
            ),
            true => Path::line(
                Point {
                    x: state.selected_point.x,
                    y: 0.0,
                },
                Point {
                    x: state.selected_point.x,
                    y: bounds.height,
                },
            ),
        };
        frame.stroke(&alpha_indicator, Stroke::default());
    }
}

// Wrapped over picker to prove to the compiler there will be no type conflict
struct PickerProgram<T>(T);

impl<T: Picker> canvas::Program<Message> for PickerProgram<T> {
    type State = PickerState;

    fn draw(
        &self,
        state: &Self::State,
        renderer: &Renderer,
        _theme: &Theme,
        bounds: iced::Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<canvas::Geometry<Renderer>> {
        let mut frame = Frame::new(renderer, bounds.size());

        self.0.draw(state, &mut frame, bounds);

        vec![frame.into_geometry()]
    }

    fn update(
        &self,
        state: &mut Self::State,
        event: &iced::Event,
        bounds: iced::Rectangle,
        cursor: mouse::Cursor,
    ) -> Option<canvas::Action<Message>> {
        // Stop evaulating if out of bounds
        let Some(relative_pos) = cursor.position_in(bounds) else {
            // Grace effect if user was holding mouse down
            if state.is_selecting
                && let Some(Point { x, y }) = cursor.position()
            {
                let x = f32::clamp(x - bounds.x, 0.0, bounds.width);
                let y = f32::clamp(y - bounds.y, 0.0, bounds.height);
                let relative_pos = Point { x, y };

                let new_colour = self.0.relative_position_to_colour(relative_pos, bounds);
                state.selected_point = relative_pos;
                state.is_selecting = false;

                return Some(Action::publish((self.0.on_commit())(new_colour)).and_capture());
            }

            state.is_selecting = false;
            return None;
        };

        match event {
            iced::Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {
                // Compute colour from relative position
                let new_colour = self.0.relative_position_to_colour(relative_pos, bounds);

                // Update internal state
                state.is_selecting = true;
                state.selected_point = relative_pos;

                // Emit on_change
                Some(Action::publish((self.0.on_change())(new_colour)).and_capture())
            }
            iced::Event::Mouse(mouse::Event::CursorMoved { position: _ }) if state.is_selecting => {
                // Compute colour from relative position
                let new_colour = self.0.relative_position_to_colour(relative_pos, bounds);

                // Update internal state
                state.selected_point = relative_pos;

                // Emit on_change
                Some(Action::publish((self.0.on_change())(new_colour)).and_capture())
            }
            iced::Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)) => {
                // Compute colour from relative position
                let final_colour = self.0.relative_position_to_colour(relative_pos, bounds);

                //  is_selected is false
                state.is_selecting = false;

                // Emit on_commit
                Some(Action::publish((self.0.on_commit())(final_colour)).and_capture())
            }

            _ => None,
        }
    }
}
