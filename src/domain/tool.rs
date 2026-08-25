#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Tool {
    Paint,
    Erase,
    #[default]
    Pan,
    Fill,
}
