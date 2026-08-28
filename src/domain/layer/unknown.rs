use iced::Rectangle;

use crate::domain::RenderTarget;

use super::LayerInnerImpl;

/// A layer whose `kind` this build doesn't recognise.
///
/// Used for a graceful degreadation in capability, if this version
/// of the application doesn't recongnize the layer content the user
/// can be informed to rectify the problem (i.e. update the app).
#[derive(Debug, Clone)]
pub struct UnknownLayer {
    pub kind: String,
    pub raw: Vec<u8>,
}

/// Not renderable - No operation for bounds and draw.
impl LayerInnerImpl for UnknownLayer {
    fn bounds(&self, _hex_size: f32) -> Option<Rectangle> {
        None
    }

    fn draw(&self, _renderer: &mut dyn RenderTarget) {}
}
