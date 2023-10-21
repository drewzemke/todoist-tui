use chrono::{NaiveDate, NaiveDateTime};
use serde::Deserialize;
use serde::Serialize;
use std::fmt::Display;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
#[serde(untagged)]
pub enum DueDate {
    Date(NaiveDate),
    DateTime(NaiveDateTime),
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct Due {
    pub date: DueDate,
}

impl Display for Due {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.date {
            DueDate::Date(date) => date.fmt(f),
            DueDate::DateTime(datetime) => datetime.fmt(f),
        }
    }
}
