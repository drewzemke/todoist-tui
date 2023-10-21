use chrono::{NaiveDate, NaiveDateTime};
use serde::Deserialize;
use serde::Serialize;
use smart_date::FlexibleDate;
use smart_date::Parsed;
use std::fmt::Display;
use std::ops::Range;

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

impl Due {
    #[must_use]
    pub fn parse_from_str(input: &str, today: NaiveDate) -> Option<(Due, Range<usize>)> {
        FlexibleDate::find_and_parse_in_str(input)
            .map(|Parsed { data, range }| (data.into_naive_date(today), range))
            .map(|(date, range)| {
                (
                    Due {
                        date: DueDate::Date(date),
                    },
                    range,
                )
            })
    }
}

impl Display for Due {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.date {
            DueDate::Date(date) => date.fmt(f),
            DueDate::DateTime(datetime) => datetime.fmt(f),
        }
    }
}
