use crate::domain::{
    RenderTarget,
    inspect::{Property, PropertyInfo},
    layer::{Inspectable, Renderable},
};

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
impl Renderable for UnknownLayer {
    fn draw(&self, _renderer: &mut dyn RenderTarget) {}
}
impl Inspectable for UnknownLayer {
    fn properties<'a>(&'a self) -> Vec<crate::domain::inspect::Property<'a>> {
        vec![
            Property::ReadOnly {
                info: None,
                display_value: Some(
                    "Unsupported layer type.\nIt will be kept as-is when you save.".to_string(),
                ),
            },
            Property::ReadOnly {
                info: Some(PropertyInfo {
                    label: "Layer kind",
                }),
                display_value: Some(self.kind.clone()),
            },
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::ports::render::MockRenderer;

    #[test]
    fn is_never_given_bounds() {
        let layer = UnknownLayer {
            kind: "future-layer-type".into(),
            raw: vec![1, 2, 3],
        };
        assert!(layer.bounds(16.0).is_none());
    }

    #[test]
    fn draw_is_a_true_no_op() {
        let layer = UnknownLayer {
            kind: "future-layer-type".into(),
            raw: vec![],
        };
        let mut renderer = MockRenderer::default();
        layer.draw(&mut renderer);

        assert!(renderer.fills.is_empty());
        assert!(renderer.strokes.is_empty());
        assert!(renderer.images.is_empty());
    }
}
