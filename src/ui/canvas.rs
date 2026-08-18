use std::cell::Cell;

use iced::{
    Element, Length, Point, Rectangle, Theme, Vector, mouse, touch,
    widget::{
        Action,
        canvas::{self, Event, Fill, Frame, Geometry, Path, Program, Stroke},
    },
};

use crate::{
    app::Message,
    domain::{HexCoord, HistoryCommand, Scene, SceneMessage, Tool, hexes_in_range},
};

pub fn canvas_panel<'a>(scene: &'a Scene) -> Element<'a, Message> {
    let hex_canvas = HexCanvas {
        scene,
        hex_size: 16.0,
    };

    iced::widget::canvas(hex_canvas)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}

pub struct HexCanvas<'a> {
    pub scene: &'a Scene,
    pub hex_size: f32,
}

#[derive(Debug)]
pub struct CanvasState {
    cache: canvas::Cache,
    // Tracks the `Layers` revision that `cache` was last drawn from. Wrapped
    // in a `Cell` so it can be updated from `Program::draw`, which only
    // hands us a `&CanvasState` (see `HexCanvas::draw`).
    cached_layers_revision: Cell<u64>,
    dragging: bool,
    last_drag_pos: Option<Point>,
    translation: Vector,
    zoom: f32,
}

impl Default for CanvasState {
    fn default() -> Self {
        Self {
            cache: Default::default(),
            cached_layers_revision: Cell::new(0),
            dragging: false,
            last_drag_pos: None,
            translation: Vector::new(0.0, 0.0),
            zoom: 1.0,
        }
    }
}

impl<'a> Program<Message> for HexCanvas<'a> {
    type State = CanvasState;

    fn draw(
        &self,
        state: &CanvasState,
        renderer: &iced::Renderer,
        theme: &Theme,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> Vec<Geometry> {
        // Invalidate render cache if the state of Layers has changed.
        let current_revision = self.scene.revision();
        if state.cached_layers_revision.get() != current_revision {
            state.cache.clear();
            state.cached_layers_revision.set(current_revision);
        }

        // Cache map drawing
        let map = state.cache.draw(renderer, bounds.size(), |frame| {
            self.draw_map(state, theme, frame, bounds);
        });

        // Get cursor pos. Otherwise just draw map by itself
        let Some(cursor_pos) = cursor.position_in(bounds) else {
            return vec![map];
        };

        // Don't draw hex indicator if user is panning
        if Tool::Pan == self.scene.tool {
            return vec![map];
        }

        let mouse = self.draw_cursor_hex(renderer, theme, bounds, state, cursor_pos);
        vec![map, mouse]
    }

    fn update(
        &self,
        state: &mut CanvasState,
        event: &Event,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> Option<Action<Message>> {
        let Some(cursor_pos) = cursor.position_in(bounds) else {
            state.dragging = false;
            state.last_drag_pos = None;
            return None;
        };

        match event {
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left))
            | Event::Touch(touch::Event::FingerPressed { .. }) => {
                state.dragging = true;
                state.last_drag_pos = Some(cursor_pos);
                let coord = self.screen_to_hex(state, cursor_pos);

                let message = match self.scene.tool {
                    Tool::Paint => Message::History(HistoryCommand::BeginTransaction(
                        SceneMessage::PaintHex(coord),
                    )),
                    Tool::Erase => Message::History(HistoryCommand::BeginTransaction(
                        SceneMessage::EraseHex(coord),
                    )),
                    Tool::Pan => return Some(Action::capture()),
                    Tool::Fill => Message::Scene(SceneMessage::FillFromHex(coord)),
                };

                Some(Action::publish(message).and_capture())
            }

            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left))
            | Event::Touch(touch::Event::FingerLifted { .. })
            | Event::Touch(touch::Event::FingerLost { .. }) => {
                state.dragging = false;
                state.last_drag_pos = None;

                Some(
                    Action::publish(Message::History(HistoryCommand::CommitTransaction))
                        .and_capture(),
                )
            }

            Event::Mouse(mouse::Event::CursorMoved { .. })
            | Event::Touch(touch::Event::FingerMoved { .. }) => {
                if !state.dragging {
                    return Some(Action::request_redraw());
                };

                let last = state.last_drag_pos;
                state.last_drag_pos = Some(cursor_pos);

                let coord = self.screen_to_hex(state, cursor_pos);

                let message = match self.scene.tool {
                    Tool::Paint => Message::Scene(SceneMessage::PaintHex(coord)),
                    Tool::Erase => Message::Scene(SceneMessage::EraseHex(coord)),
                    // Bucket fill is explicitly disabled while dragging
                    // This is to avoid triggering epilieptic seizures
                    Tool::Fill => return None,
                    Tool::Pan => match last {
                        None => return None,
                        Some(last) => {
                            state.translation.x += cursor_pos.x - last.x;
                            state.translation.y += cursor_pos.y - last.y;
                            state.cache.clear();
                            return Some(Action::request_redraw().and_capture());
                        }
                    },
                };

                Some(Action::publish(message).and_capture())
            }

            Event::Mouse(mouse::Event::WheelScrolled { delta }) => {
                let delta = match delta {
                    mouse::ScrollDelta::Lines { x, y } => (x + y) * 20.0,
                    mouse::ScrollDelta::Pixels { x, y } => x + y,
                };
                state.zoom = f32::clamp(state.zoom + delta * 0.01, 0.4, 10.0);
                state.cache.clear();
                Some(Action::request_redraw().and_capture())
            }

            _ => None,
        }
    }

    fn mouse_interaction(
        &self,
        state: &CanvasState,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> mouse::Interaction {
        if !cursor.is_over(bounds) {
            return mouse::Interaction::Idle;
        }

        match self.scene.tool {
            Tool::Pan if state.dragging => mouse::Interaction::Grabbing,
            Tool::Pan => mouse::Interaction::Grab,
            Tool::Paint => mouse::Interaction::Crosshair,
            Tool::Erase => mouse::Interaction::Crosshair,
            Tool::Fill => mouse::Interaction::Crosshair,
        }
    }
}

impl<'a> HexCanvas<'a> {
    fn draw_cursor_hex(
        &self,
        renderer: &iced::Renderer,
        theme: &Theme,
        bounds: Rectangle,
        state: &CanvasState,
        cursor_pos: Point,
    ) -> Geometry {
        let mut frame = Frame::new(renderer, bounds.size());

        let hex = self.screen_to_hex(state, cursor_pos);
        let coord = hex.to_cartesian() * self.hex_size * state.zoom + state.translation;

        frame.translate(coord);
        frame.scale(state.zoom);

        let stroke = Stroke {
            style: canvas::Style::Solid(theme.extended_palette().primary.base.color),
            width: 2.0 / state.zoom,
            ..Stroke::default()
        };

        frame.stroke(&self.hex_path(), stroke);
        frame.into_geometry()
    }

    fn draw_map(&self, state: &CanvasState, theme: &Theme, frame: &mut Frame, bounds: Rectangle) {
        frame.translate(state.translation);
        frame.scale(state.zoom);

        // Compute hex bounds in map-space for culling
        let inv_zoom = 1.0 / state.zoom;
        let inv_hex_w = 1.0 / (self.hex_size * 1.5);
        let inv_hex_h = 1.0 / (self.hex_size * (3.0_f32).sqrt());

        let col_min = (-state.translation.x * inv_hex_w * inv_zoom).floor() as i32;
        let col_max = col_min + (bounds.width * inv_hex_w * inv_zoom).ceil() as i32;

        let row_min = (-state.translation.y * inv_hex_h * inv_zoom).floor() as i32;
        let row_max = row_min + (bounds.height * inv_hex_h * inv_zoom).ceil() as i32;

        let hex_path = &self.hex_path();

        // Draw grid layers
        for layer in self.scene.get_visible_layers() {
            // Column / Row maximum increased to full cover screen
            let coords = hexes_in_range(col_min, col_max + 1, row_min, row_max + 1);

            layer.draw(frame, coords, |frame, hex, colour| {
                let centre = hex.to_cartesian() * self.hex_size;
                frame.with_save(|frame| {
                    frame.translate(centre);
                    frame.fill(hex_path, Fill::from(colour));
                })
            });
        }

        // Draw grid overlay
        let grid_stroke = Stroke {
            style: canvas::Style::Solid(
                theme
                    .extended_palette()
                    .background
                    .base
                    .text
                    .scale_alpha(0.1),
            ),
            width: 1.0,
            ..Stroke::default()
        };

        let coords = hexes_in_range(col_min, col_max, row_min, row_max);
        for hex in coords {
            let centre = hex.to_cartesian() * self.hex_size;
            frame.with_save(|frame| {
                frame.translate(centre);
                frame.stroke(&hex_path, grid_stroke);
            })
        }
    }

    fn screen_to_hex(&self, state: &CanvasState, screen: Point) -> HexCoord {
        let translation = state.translation;
        let zoom = state.zoom;

        let x = (screen.x - translation.x) / zoom;
        let y = (screen.y - translation.y) / zoom;
        let map_vec = Vector { x, y };

        HexCoord::from_cartesian(map_vec / self.hex_size)
    }

    fn hex_path(&self) -> Path {
        let mut builder = canvas::path::Builder::new();
        for i in 0..6 {
            let angle = std::f32::consts::PI / 180.0 * (60.0 * i as f32);
            let px = self.hex_size * angle.cos();
            let py = self.hex_size * angle.sin();
            if i == 0 {
                builder.move_to(Point::new(px, py));
            } else {
                builder.line_to(Point::new(px, py));
            }
        }
        builder.close();
        builder.build()
    }
}
