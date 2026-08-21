use crate::domain::layer_inner::LayerInner;

#[derive(Debug, Clone)]
pub struct Layer {
    pub name: String,
    pub visible: bool,

    pub inner: LayerInner,
}
