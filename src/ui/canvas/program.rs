//! Implements [`shader::Program`] for [`HexCanvas`].
//!
//! `update` dispatches individual gestures using [`CanvasEvent`],
//! so that each can be reasoned about independantly.

use std::sync::Arc;

use iced::{
    Point, Rectangle, Vector, mouse, touch,
    widget::{Action, shader},
};

use crate::domain::{RenderTarget, Tool, layer::LayerInnerImpl, layer::overlay::HexGridOverlay};

use super::CanvasEvent;
use super::gpu::{DrawCommand, HexMapPrimitive};
use super::render_target::GpuRenderTarget;
use super::state::CanvasState;
use super::{CURSOR_HEX_COLOR, HEX_SIZE, HexCanvas};

impl<'a> shader::Program<CanvasEvent> for HexCanvas<'a> {
    type State = CanvasState;
    type Primitive = HexMapPrimitive;

    fn draw(
        &self,
        state: &CanvasState,
        cursor: mouse::Cursor,
        bounds: Rectangle,
    ) -> HexMapPrimitive {
        let current_revision = self.scene.revision();

        let base =
            state.base_commands(current_revision, || self.build_base_commands(state, bounds));
        let overlay = self.build_overlay_commands(state, cursor.position_in(bounds), bounds);

        HexMapPrimitive {
            base,
            overlay: Arc::new(overlay),
            translation: state.camera.translation,
            zoom: state.camera.zoom,
        }
    }

    fn update(
        &self,
        state: &mut CanvasState,
        event: &iced::Event,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> Option<Action<CanvasEvent>> {
        let Some(cursor_pos) = cursor.position_in(bounds) else {
            state.drag_anchor = None;
            return None;
        };

        match event {
            iced::Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left))
            | iced::Event::Touch(touch::Event::FingerPressed { .. }) => {
                self.handle_press(state, cursor_pos)
            }

            iced::Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left))
            | iced::Event::Touch(touch::Event::FingerLifted { .. })
            | iced::Event::Touch(touch::Event::FingerLost { .. }) => self.handle_release(state),

            iced::Event::Mouse(mouse::Event::CursorMoved { .. })
            | iced::Event::Touch(touch::Event::FingerMoved { .. }) => {
                self.handle_move(state, cursor_pos)
            }

            iced::Event::Mouse(mouse::Event::WheelScrolled { delta }) => {
                self.handle_scroll(state, *delta)
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
            return mouse::Interaction::None;
        }

        match self.tool {
            Tool::Pan if state.is_dragging() => mouse::Interaction::Grabbing,
            Tool::Pan => mouse::Interaction::Grab,
            Tool::Paint | Tool::Erase | Tool::Fill => mouse::Interaction::Crosshair,
        }
    }
}

impl<'a> HexCanvas<'a> {
    fn handle_press(
        &self,
        state: &mut CanvasState,
        cursor_pos: Point,
    ) -> Option<Action<CanvasEvent>> {
        state.drag_anchor = Some(cursor_pos);

        if self.tool == Tool::Pan {
            return Some(Action::capture());
        }

        let coord = state.camera.screen_to_hex(cursor_pos);
        Some(Action::publish(CanvasEvent::PointerPressed { at: coord }).and_capture())
    }

    fn handle_release(&self, state: &mut CanvasState) -> Option<Action<CanvasEvent>> {
        state.drag_anchor = None;

        if self.tool == Tool::Pan {
            return None;
        }

        Some(Action::publish(CanvasEvent::PointerReleased).and_capture())
    }

    fn handle_move(
        &self,
        state: &mut CanvasState,
        cursor_pos: Point,
    ) -> Option<Action<CanvasEvent>> {
        // Not dragging: just keep the cursor-hover overlay redrawing.
        let Some(last) = state.drag_anchor else {
            return Some(Action::request_redraw());
        };
        state.drag_anchor = Some(cursor_pos);

        if self.tool == Tool::Pan {
            state
                .camera
                .pan_by(Vector::new(cursor_pos.x - last.x, cursor_pos.y - last.y));
            return Some(Action::request_redraw().and_capture());
        }

        let from = state.camera.screen_to_hex(last);
        let to = state.camera.screen_to_hex(cursor_pos);
        Some(Action::publish(CanvasEvent::PointerMoved { from, to }).and_capture())
    }

    fn handle_scroll(
        &self,
        state: &mut CanvasState,
        delta: mouse::ScrollDelta,
    ) -> Option<Action<CanvasEvent>> {
        let amount = match delta {
            mouse::ScrollDelta::Lines { x, y } => (x + y) * 20.0,
            mouse::ScrollDelta::Pixels { x, y } => x + y,
        };
        state.camera.zoom_by(amount * 0.01);

        Some(Action::request_redraw().and_capture())
    }

    /// Draw commands for every visible layer, in world space. Cached by
    /// `draw` and only rebuilt when `Scene::revision()` changes.
    fn build_base_commands(&self, state: &CanvasState, bounds: Rectangle) -> Vec<DrawCommand> {
        let mut target =
            GpuRenderTarget::new(state.camera.visible_hex_bounds(bounds), &self.scene.assets);

        for layer in self.scene.get_visible_layers() {
            layer.draw(&mut target);
        }

        target.finish()
    }

    /// Draw commands for the hex-grid lines and cursor-hover outline.
    /// Rebuilt every frame so the overlay stays responsive to the cursor.
    fn build_overlay_commands(
        &self,
        state: &CanvasState,
        cursor_pos: Option<Point>,
        bounds: Rectangle,
    ) -> Vec<DrawCommand> {
        let mut target =
            GpuRenderTarget::new(state.camera.visible_hex_bounds(bounds), &self.scene.assets);

        let grid = HexGridOverlay::new_light(1.5 / state.camera.zoom);
        grid.draw(&mut target);

        if let Some(cursor_pos) = cursor_pos {
            let hex = state.camera.screen_to_hex(cursor_pos);
            let world = hex.to_cartesian() * HEX_SIZE;
            let center = Point::new(world.x, world.y);

            target.stroke_polygon(&center, CURSOR_HEX_COLOR, 2.0 / state.camera.zoom);
        }

        target.finish()
    }
}
