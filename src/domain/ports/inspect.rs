use std::ops::RangeInclusive;

use crate::domain::{EditCommand, id::LayerId};

pub trait Inspectable {
    fn properties<'a>(&'a self) -> Vec<Property<'a>>;
}

pub enum Property<'a> {
    Text(TextProperty<'a>),
    Bool(BoolProperty<'a>),
    Float(FloatProperty<'a>),
    Integer(IntegerProperty<'a>),
    BoundedFloat(BoundedFloatProperty<'a>),
    BoundedInteger(BoundedIntegerProperty<'a>),
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

pub struct FloatProperty<'a> {
    pub name: &'a str,
    pub value: f64,
    pub on_submit: OnSubmit<'a, f64>,
}

pub struct IntegerProperty<'a> {
    pub name: &'a str,
    pub value: u64,
    pub on_submit: OnSubmit<'a, u64>,
}

pub struct BoundedFloatProperty<'a> {
    pub name: &'a str,
    pub value: f64,
    pub range: RangeInclusive<f64>,
    pub on_submit: OnSubmit<'a, f64>,
}

pub struct BoundedIntegerProperty<'a> {
    pub name: &'a str,
    pub value: u64,
    pub range: RangeInclusive<u64>,
    pub on_submit: OnSubmit<'a, u64>,
}
