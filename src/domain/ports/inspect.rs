use crate::domain::{EditCommand, id::LayerId};

pub trait Inspectable {
    fn properties<'a>(&'a self) -> Vec<Property<'a>>;
}

pub enum Property<'a> {
    Text(TextProperty<'a>),
    Bool(BoolProperty<'a>),
}

type OnSubmit<'a, T> = Box<dyn Fn(T, LayerId) -> Box<dyn EditCommand> + 'a>;

pub struct TextProperty<'a> {
    pub name: &'a str,
    pub value: &'a str,
    pub on_submit: OnSubmit<'a, String>,
}

pub struct BoolProperty<'a> {
    pub name: &'a str,
    pub value: bool,
    pub on_submit: OnSubmit<'a, bool>,
}
