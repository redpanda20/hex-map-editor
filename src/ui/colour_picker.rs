#! Colour picker widget

use iced::{
    Alignment, Color, Element, Length, Point, Renderer, Theme, mouse,
    widget::{
        Action,
        canvas::{self, Frame, Path, Stroke},
        column, container, row, slider, text, tooltip,
    },
};

use crate::{app::Message, domain::colour::Hsva};

pub fn colour_picker<'a>(
    colour: Color,
    on_change: impl Fn(Color) -> Message + 'a + Copy,
    on_commit: impl Fn(Color) -> Message + 'a + Copy,
) -> Element<'a, Message> {
    let Hsva {
        hue,
        saturation,
        value,
        alpha,
    } = Hsva::from(colour);

    let hue_colour: Color = Hsva {
        hue,
        saturation: 1.0,
        value: 1.0,
        alpha,
    }
    .into();

    let sv_picker = SaturationValuePicker {
        alpha,
        hue,
        hue_colour,
        on_change: Box::new(on_change),
        on_commit: Box::new(on_commit),
    };

    let hue_picker = HuePicker {
        alpha,
        saturation,
        value,
        on_change: Box::new(on_change),
        on_commit: Box::new(on_commit),
    };

    // TODO: Replace the alpha slider with a graphical version
    let alpha_slider = slider(0.0..=1.0, alpha, move |new_alpha| {
        let mut new_colour = colour.clone();
        new_colour.a = new_alpha;
        (on_change)(new_colour)
    })
    .on_release((on_commit)(colour))
    .step(0.01);
    let alpha_slider = row![
        tooltip(
            text("A"),
            container("Alpha")
                .padding(4.0)
                .style(container::bordered_box),
            tooltip::Position::Left
        ),
        alpha_slider
    ]
    .align_y(Alignment::Center)
    .spacing(8.0);

    column![
        iced::widget::canvas(sv_picker)
            .width(Length::Fill)
            .height(Length::Fill),
        iced::widget::canvas(hue_picker)
            .width(Length::Fill)
            .height(Length::Fixed(20.0)),
        alpha_slider
    ]
    .spacing(8.0)
    .into()
}

/// Saturation value square
pub struct SaturationValuePicker<'a> {
    // Alpha
    alpha: f32,
    // Hue in degrees
    hue: f32,
    // Full saturation, Full value colour of the correct hue
    hue_colour: Color,
    on_change: Box<dyn Fn(Color) -> Message + 'a>,
    on_commit: Box<dyn Fn(Color) -> Message + 'a>,
}

#[derive(Debug, Default)]
pub struct PickerState {
    is_selecting: bool,
    selected_point: Point,
}

impl<'a> canvas::Program<Message> for SaturationValuePicker<'a> {
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

        self.draw(state, &mut frame, bounds);

        vec![frame.into_geometry()]
    }

    fn update(
        &self,
        state: &mut Self::State,
        event: &iced::Event,
        bounds: iced::Rectangle,
        cursor: mouse::Cursor,
    ) -> Option<canvas::Action<Message>> {
        // Check if mouse is out of bounds first
        let Some(relative_pos) = cursor.position_in(bounds) else {
            state.is_selecting = false;
            return None;
        };

        // iced::Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left))
        match event {
            iced::Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {
                // Compute colour from relative position
                let new_colour = self.relative_position_to_colour(relative_pos, bounds);

                // Update internal state
                state.is_selecting = true;
                state.selected_point = relative_pos;

                // Emit on_change
                Some(Action::publish((self.on_change)(new_colour)).and_capture())
            }
            iced::Event::Mouse(mouse::Event::CursorMoved { position: _ }) if state.is_selecting => {
                // Compute colour from relative position
                let new_colour = self.relative_position_to_colour(relative_pos, bounds);

                // Update internal state
                state.selected_point = relative_pos;

                // Emit on_change
                Some(Action::publish((self.on_change)(new_colour)).and_capture())
            }
            iced::Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)) => {
                // Compute colour from relative position
                let final_colour = self.relative_position_to_colour(relative_pos, bounds);

                //  is_selected is false
                state.is_selecting = false;

                // Emit on_commit
                Some(Action::publish((self.on_commit)(final_colour)).and_capture())
            }

            _ => None,
        }
    }
}

impl<'a> SaturationValuePicker<'a> {
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
            .add_stop(1.0, self.hue_colour);

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
}

/// Hue colour line
pub struct HuePicker<'a> {
    saturation: f32,
    value: f32,
    alpha: f32,
    on_change: Box<dyn Fn(Color) -> Message + 'a>,
    on_commit: Box<dyn Fn(Color) -> Message + 'a>,
}

impl<'a> HuePicker<'a> {
    fn relative_position_to_colour(&self, relative_pos: Point, bounds: iced::Rectangle) -> Color {
        let hue = (relative_pos.x / bounds.width) * 360.0;

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
        let right = Point {
            x: bounds.width,
            y: 0.0,
        };

        let hue_gradient = canvas::gradient::Linear::new(origin, right)
            .add_stop(0.00, Color::from_rgba(1.0, 0.0, 0.0, 1.0))
            .add_stop(0.16, Color::from_rgba(1.0, 1.0, 0.0, 1.0))
            .add_stop(0.33, Color::from_rgba(0.0, 1.0, 0.0, 1.0))
            .add_stop(0.50, Color::from_rgba(0.0, 1.0, 1.0, 1.0))
            .add_stop(0.66, Color::from_rgba(0.0, 0.0, 1.0, 1.0))
            .add_stop(0.66, Color::from_rgba(0.0, 0.0, 1.0, 1.0))
            .add_stop(0.83, Color::from_rgba(1.0, 0.0, 1.0, 1.0))
            .add_stop(1.00, Color::from_rgba(1.0, 0.0, 0.0, 1.0));

        frame.fill_rectangle(Point::ORIGIN, bounds.size(), hue_gradient);

        let hue_indicator = Path::line(
            Point {
                x: state.selected_point.x,
                y: 0.0,
            },
            Point {
                x: state.selected_point.x,
                y: bounds.height,
            },
        );
        frame.stroke(&hue_indicator, Stroke::default());
    }
}

impl<'a> canvas::Program<Message> for HuePicker<'a> {
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

        self.draw(state, &mut frame, bounds);

        vec![frame.into_geometry()]
    }

    fn update(
        &self,
        state: &mut Self::State,
        event: &iced::Event,
        bounds: iced::Rectangle,
        cursor: mouse::Cursor,
    ) -> Option<canvas::Action<Message>> {
        // Check if mouse is out of bounds first
        let Some(relative_pos) = cursor.position_in(bounds) else {
            state.is_selecting = false;
            return None;
        };

        // iced::Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left))
        match event {
            iced::Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {
                // Compute colour from relative position
                let new_colour = self.relative_position_to_colour(relative_pos, bounds);

                // Update internal state
                state.is_selecting = true;
                state.selected_point = relative_pos;

                // Emit on_change
                Some(Action::publish((self.on_change)(new_colour)).and_capture())
            }
            iced::Event::Mouse(mouse::Event::CursorMoved { position: _ }) if state.is_selecting => {
                // Compute colour from relative position
                let new_colour = self.relative_position_to_colour(relative_pos, bounds);

                // Update internal state
                state.selected_point = relative_pos;

                // Emit on_change
                Some(Action::publish((self.on_change)(new_colour)).and_capture())
            }
            iced::Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)) => {
                // Compute colour from relative position
                let final_colour = self.relative_position_to_colour(relative_pos, bounds);

                //  is_selected is false
                state.is_selecting = false;

                // Emit on_commit
                Some(Action::publish((self.on_commit)(final_colour)).and_capture())
            }

            _ => None,
        }
    }
}
