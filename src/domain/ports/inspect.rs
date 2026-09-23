use std::ops::RangeInclusive;

use iced::Color;

use crate::domain::{EditCommand, id::LayerId};

pub trait Inspectable {
    fn properties<'a>(&'a self) -> Vec<Property<'a>>;
}

pub enum Property<'a> {
    Action {
        info: PropertyInfo,
        value: Option<String>,
        action_hint: ActionHint,
        action: Box<dyn Fn(LayerId) -> Box<dyn EditCommand> + 'a>,
    },

    Text {
        info: PropertyInfo,
        value: &'a str,
        on_submit: OnSubmit<'a, String>,
    },
    Float {
        info: PropertyInfo,
        value: f64,
        on_submit: OnSubmit<'a, f64>,
    },
    Integer {
        info: PropertyInfo,
        value: u64,
        on_submit: OnSubmit<'a, u64>,
    },
    Colour {
        info: PropertyInfo,
        value: Color,
        on_submit: OnSubmit<'a, Color>,
    },

    BoundedFloat {
        info: PropertyInfo,
        value: f64,
        range: RangeInclusive<f64>,
        on_submit: OnSubmit<'a, f64>,
    },
    BoundedInteger {
        info: PropertyInfo,
        value: u64,
        range: RangeInclusive<u64>,
        on_submit: OnSubmit<'a, u64>,
    },
}

type OnSubmit<'a, T> = Box<dyn Fn(T, LayerId) -> Box<dyn EditCommand> + 'a>;

pub struct PropertyInfo {
    // pub id: &'static str,
    pub label: &'static str,
    // pub description: Option<&'static str>,
}

pub enum ActionHint {
    Text(&'static str),
    Refresh,
}
