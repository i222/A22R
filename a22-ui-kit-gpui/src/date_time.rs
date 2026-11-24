use chrono::{Datelike, NaiveTime, Timelike};
use gpui_kit::{
    component::{
        calendar::Date as KitDate,
        date_picker::{
            DatePicker as KitDatePicker, DatePickerEvent as KitDatePickerEvent, DatePickerState,
        },
        time_field::{
            HourCycle, TimeField as KitTimeField, TimeFieldEvent, TimeFieldState, TimePrecision,
        },
        Sizable,
    },
    div,
    prelude::*,
    px, Context, Entity, EventEmitter, IntoElement, Render, SharedString, Window,
};

use crate::{
    theme::{theme, Size},
    value::{Date, Signal, Time},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DatePickerEvent {
    Selected(Date),
}

pub struct DatePicker {
    state: Option<Entity<DatePickerState>>,
    value: Option<Date>,
    pending_sync: bool,
    placeholder: SharedString,
    label: Option<SharedString>,
    size: Size,
}

impl EventEmitter<DatePickerEvent> for DatePicker {}

impl DatePicker {
    pub fn new(_: &mut Context<Self>) -> Self {
        Self {
            state: None,
            value: None,
            pending_sync: false,
            placeholder: SharedString::default(),
            label: None,
            size: Size::Xs,
        }
    }

    pub fn placeholder(mut self, placeholder: impl Into<SharedString>) -> Self {
        self.placeholder = placeholder.into();
        self
    }

    pub fn label(mut self, label: impl Into<SharedString>) -> Self {
        self.label = Some(label.into());
        self
    }

    pub fn size(mut self, size: Size) -> Self {
        self.size = size;
        self
    }

    pub fn value(mut self, value: Date) -> Self {
        self.value = Some(value);
        self.pending_sync = true;
        self
    }

    pub fn set_value(&mut self, value: Option<Date>, cx: &mut Context<Self>) {
        self.value = value;
        self.pending_sync = true;
        cx.notify();
    }

    pub fn bind<C: ?Sized>(_entity: &Entity<Self>, _signal: &Signal<Option<Date>>, _cx: &mut C) {}

    fn ensure_state(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.state.is_some() {
            return;
        }
        let state = cx.new(|cx| DatePickerState::new(window, cx));
        cx.subscribe(&state, move |picker, _, event: &KitDatePickerEvent, cx| {
            let KitDatePickerEvent::Change(value) = event;
            let Some(date) = value.start().map(|value| value.date()) else {
                return;
            };
            if let Some(date) = Date::new(date.year(), date.month(), date.day()) {
                picker.value = Some(date);
                cx.emit(DatePickerEvent::Selected(date));
                cx.notify();
            }
        })
        .detach();
        self.state = Some(state);
    }
}

impl Render for DatePicker {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        self.ensure_state(window, cx);
        let state = self.state.as_ref().expect("date state initialized").clone();
        if self.pending_sync {
            let date = self
                .value
                .map(|value| KitDate::Single(Some(value.naive())))
                .unwrap_or(KitDate::Single(None));
            state.update(cx, |state, cx| state.set_date(date, window, cx));
            self.pending_sync = false;
        }

        let picker = KitDatePicker::new(&state)
            .placeholder(self.placeholder.clone())
            .with_size(self.size.kit())
            .w(px(160.0));
        let label = self.label.clone();
        let tokens = theme(cx);
        div()
            .flex()
            .flex_col()
            .gap(px(4.0))
            .when_some(label, |this, label| {
                this.child(
                    div()
                        .text_color(tokens.text().hsla())
                        .text_size(px(tokens.font_size(Size::Sm)))
                        .child(label),
                )
            })
            .child(picker)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TimePickerEvent(pub Time);

pub struct TimePicker {
    state: Option<Entity<TimeFieldState>>,
    value: Option<Time>,
    pending_sync: bool,
    placeholder: SharedString,
    label: Option<SharedString>,
    size: Size,
}

impl EventEmitter<TimePickerEvent> for TimePicker {}

impl TimePicker {
    pub fn new(_: &mut Context<Self>) -> Self {
        Self {
            state: None,
            value: None,
            pending_sync: false,
            placeholder: SharedString::default(),
            label: None,
            size: Size::Xs,
        }
    }

    pub fn placeholder(mut self, placeholder: impl Into<SharedString>) -> Self {
        self.placeholder = placeholder.into();
        self
    }

    pub fn label(mut self, label: impl Into<SharedString>) -> Self {
        self.label = Some(label.into());
        self
    }

    pub fn size(mut self, size: Size) -> Self {
        self.size = size;
        self
    }

    pub fn twenty_four_hour(self) -> Self {
        self
    }

    pub fn value(mut self, value: Time) -> Self {
        self.value = Some(value);
        self.pending_sync = true;
        self
    }

    pub fn set_value(&mut self, value: Option<Time>, cx: &mut Context<Self>) {
        self.value = value;
        self.pending_sync = true;
        cx.notify();
    }

    pub fn bind<C: ?Sized>(_entity: &Entity<Self>, _signal: &Signal<Option<Time>>, _cx: &mut C) {}

    fn ensure_state(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.state.is_some() {
            return;
        }
        let state = cx.new(|cx| {
            TimeFieldState::new(window, cx)
                .precision(TimePrecision::Minute)
                .hour_cycle(HourCycle::H23)
        });
        cx.subscribe(&state, move |picker, _, event: &TimeFieldEvent, cx| {
            let TimeFieldEvent::Change(value) = event;
            if let Some(time) = Time::new(value.hour(), value.minute()) {
                picker.value = Some(time);
                cx.emit(TimePickerEvent(time));
                cx.notify();
            }
        })
        .detach();
        self.state = Some(state);
    }
}

impl Render for TimePicker {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        self.ensure_state(window, cx);
        let state = self.state.as_ref().expect("time state initialized").clone();
        if self.pending_sync {
            let value = self.value.map(Time::naive).unwrap_or(NaiveTime::MIN);
            state.update(cx, |state, cx| state.set_time(value, window, cx));
            self.pending_sync = false;
        }

        let picker = KitTimeField::new(&state)
            .with_size(self.size.kit())
            .w(px(112.0));
        let label = self.label.clone();
        let tokens = theme(cx);
        div()
            .flex()
            .flex_col()
            .gap(px(4.0))
            .when_some(label, |this, label| {
                this.child(
                    div()
                        .text_color(tokens.text().hsla())
                        .text_size(px(tokens.font_size(Size::Sm)))
                        .child(label),
                )
            })
            .child(div().child(picker))
    }
}
