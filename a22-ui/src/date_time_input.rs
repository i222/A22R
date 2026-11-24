use a22_ui_kit_gpui::{
    Date, DatePicker, DatePickerEvent, Signal, Size, Time, TimePicker, TimePickerEvent,
};
use gpui_kit::{
    div, prelude::*, px, Context, Entity, EventEmitter, IntoElement, SharedString, Window,
};

/// The combined date and time value emitted by [`DateTimeInput`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DateTimeInputEvent(pub Option<(Date, Time)>);

/// A product-level date/time control composed from the current UI backend.
///
/// The application only sees this event and the programmatic `set_value`
/// method; it does not depend on the backend's picker composition.
pub struct DateTimeInput {
    date_picker: Entity<DatePicker>,
    time_picker: Entity<TimePicker>,
    date_signal: Signal<Option<Date>>,
    time_signal: Signal<Option<Time>>,
    date: Option<Date>,
    time: Option<Time>,
    label: Option<SharedString>,
}

impl EventEmitter<DateTimeInputEvent> for DateTimeInput {}

impl DateTimeInput {
    pub fn new(
        cx: &mut Context<Self>,
        date_placeholder: impl Into<SharedString>,
        time_placeholder: impl Into<SharedString>,
    ) -> Self {
        let date_signal = Signal::new(cx, None);
        let time_signal = Signal::new(cx, None);
        let date_picker = cx.new(|cx| {
            DatePicker::new(cx)
                .placeholder(date_placeholder)
                .size(Size::Xs)
        });
        let time_picker = cx.new(|cx| {
            TimePicker::new(cx)
                .placeholder(time_placeholder)
                .twenty_four_hour()
                .size(Size::Xs)
        });
        DatePicker::bind(&date_picker, &date_signal, cx);
        TimePicker::bind(&time_picker, &time_signal, cx);

        cx.subscribe(
            &date_picker,
            move |input, _, event: &DatePickerEvent, cx| {
                let DatePickerEvent::Selected(date) = event;
                let date = *date;
                input.date = Some(date);
                if input.time.is_none() {
                    input.time = Time::new(0, 0);
                    input.time_signal.set(cx, input.time);
                }
                input.date_signal.set(cx, input.date);
                input.emit_change(cx);
            },
        )
        .detach();

        cx.subscribe(
            &time_picker,
            move |input, _, event: &TimePickerEvent, cx| {
                let time = event.0;
                input.time = Some(time);
                input.time_signal.set(cx, input.time);
                input.emit_change(cx);
            },
        )
        .detach();

        Self {
            date_picker,
            time_picker,
            date_signal,
            time_signal,
            date: None,
            time: None,
            label: None,
        }
    }

    pub fn label(mut self, label: impl Into<SharedString>) -> Self {
        self.label = Some(label.into());
        self
    }

    pub fn set_value(&mut self, value: Option<(Date, Time)>, cx: &mut Context<Self>) {
        self.date = value.map(|(date, _)| date);
        self.time = value.map(|(_, time)| time);
        let date = self.date;
        let time = self.time;
        self.date_signal.set(cx, date);
        self.time_signal.set(cx, time);
        self.date_picker
            .update(cx, |picker, cx| picker.set_value(date, cx));
        self.time_picker
            .update(cx, |picker, cx| picker.set_value(time, cx));
        cx.notify();
    }

    pub fn value(&self) -> Option<(Date, Time)> {
        Some((self.date?, self.time?))
    }

    fn emit_change(&self, cx: &mut Context<Self>) {
        cx.emit(DateTimeInputEvent(self.value()));
        cx.notify();
    }
}

impl Render for DateTimeInput {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        let controls = div()
            .flex()
            .items_end()
            .gap(px(6.0))
            .child(self.date_picker.clone())
            .child(self.time_picker.clone());
        if let Some(label) = self.label.clone() {
            div()
                .flex()
                .flex_col()
                .gap(px(4.0))
                .child(label)
                .child(controls)
        } else {
            controls
        }
    }
}
