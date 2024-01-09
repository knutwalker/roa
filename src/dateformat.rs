use std::fmt;

use chrono::NaiveDate as Date;
use human_date_parser::ParseResult;
use kommandozeile::{color_eyre::eyre::eyre, Result};
use serde::{de, Deserializer, Serializer};

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub enum PublishDate {
    #[default]
    Draft,
    Date(Date),
}

pub fn parse(value: &str) -> Result<PublishDate> {
    if value.is_empty() || value.eq_ignore_ascii_case("draft") || value.eq_ignore_ascii_case("none")
    {
        return Ok(PublishDate::Draft);
    }
    match human_date_parser::from_human_time(value) {
        Ok(ParseResult::Date(date)) => Ok(PublishDate::Date(date)),
        Ok(ParseResult::DateTime(date)) => Ok(PublishDate::Date(date.date_naive())),
        Ok(ParseResult::Time(_)) => Err(eyre!("need a date, not just a time")),
        Err(_) => Ok(PublishDate::Date(Date::parse_from_str(value, "%Y-%m-%d")?)),
    }
}

#[allow(clippy::trivially_copy_pass_by_ref)]
pub fn serialize<S: Serializer>(
    option: &Option<PublishDate>,
    serializer: S,
) -> Result<S::Ok, S::Error> {
    match option {
        Some(PublishDate::Date(date)) => serializer.collect_str(&date.format("%Y-%m-%d")),
        Some(PublishDate::Draft) => serializer.serialize_str(""),
        None => serializer.serialize_none(),
    }
}

pub fn deserialize<'a, D: Deserializer<'a>>(
    deserializer: D,
) -> Result<Option<PublishDate>, D::Error> {
    deserializer.deserialize_option(Visitor::<false>)
}

struct Visitor<const INNER: bool>;

impl<'de> de::Visitor<'de> for Visitor<false> {
    type Value = Option<PublishDate>;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("an optional ISO 8601 date")
    }

    fn visit_some<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_any(Visitor::<true>).map(Some)
    }

    fn visit_unit<E: de::Error>(self) -> Result<Option<PublishDate>, E> {
        Ok(None)
    }

    fn visit_none<E: de::Error>(self) -> Result<Option<PublishDate>, E> {
        Ok(None)
    }
}

impl<'de> de::Visitor<'de> for Visitor<true> {
    type Value = PublishDate;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("an ISO 8601 date")
    }

    fn visit_str<E: de::Error>(self, value: &str) -> Result<PublishDate, E> {
        parse(value).map_err(E::custom)
    }
}
