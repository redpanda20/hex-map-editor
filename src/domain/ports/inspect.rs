use std::ops::RangeInclusive;

use iced::Color;

use crate::domain::{EditCommand, assets::FileKind, id::LayerId};

pub enum Property<'a> {
    Group {
        info: Option<PropertyInfo>,
        children: Vec<Property<'a>>,
    },

    ReadOnly {
        info: Option<PropertyInfo>,
        display_value: Option<String>,
    },
    Action {
        info: PropertyInfo,
        display_value: Option<String>,
        action_hint: ActionHint,
        action: Box<dyn Fn(LayerId) -> Box<dyn EditCommand> + 'a>,
    },
    File {
        info: PropertyInfo,
        display_value: Option<String>,
        kind: FileKind,
    },

    Boolean {
        info: PropertyInfo,
        value: bool,
        on_submit: OnSubmit<'a, bool>,
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
