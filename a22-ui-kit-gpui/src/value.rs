use std::{cell::RefCell, rc::Rc};

use chrono::{Datelike, NaiveDate, NaiveTime};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Date {
    year: i32,
    month: u32,
    day: u32,
}

impl Date {
    pub fn new(year: i32, month: u32, day: u32) -> Option<Self> {
        NaiveDate::from_ymd_opt(year, month, day).map(|_| Self { year, month, day })
    }

    pub fn today() -> Self {
        let date = chrono::Local::now().date_naive();
        Self {
            year: date.year(),
            month: date.month(),
            day: date.day(),
        }
    }

    pub fn year(self) -> i32 {
        self.year
    }

    pub fn month(self) -> u32 {
        self.month
    }

    pub fn day(self) -> u32 {
        self.day
    }

    pub(crate) fn naive(self) -> NaiveDate {
        NaiveDate::from_ymd_opt(self.year, self.month, self.day)
            .expect("Date invariant guarantees a valid date")
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Time {
    hour: u32,
    minute: u32,
}

impl Time {
    pub fn new(hour: u32, minute: u32) -> Option<Self> {
        NaiveTime::from_hms_opt(hour, minute, 0).map(|_| Self { hour, minute })
    }

    pub fn hour(self) -> u32 {
        self.hour
    }

    pub fn minute(self) -> u32 {
        self.minute
    }

    pub(crate) fn naive(self) -> NaiveTime {
        NaiveTime::from_hms_opt(self.hour, self.minute, 0)
            .expect("Time invariant guarantees a valid time")
    }
}

#[derive(Clone)]
pub struct Signal<T>(Rc<RefCell<T>>);

impl<T> Signal<T> {
    pub fn new<C: ?Sized>(_: &mut C, value: T) -> Self {
        Self(Rc::new(RefCell::new(value)))
    }

    pub fn set<C: ?Sized>(&self, _: &mut C, value: T) {
        *self.0.borrow_mut() = value;
    }

    pub fn set_if_changed<C: ?Sized>(&self, cx: &mut C, value: T)
    where
        T: PartialEq,
    {
        if *self.0.borrow() != value {
            self.set(cx, value);
        }
    }
}
