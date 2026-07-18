use askama::Values;
use time::{
   OffsetDateTime,
   UtcOffset,
   format_description::well_known::Rfc3339,
};

pub fn ui_datetime(value: &OffsetDateTime, _: &dyn Values) -> askama::Result<String> {
   let value = value.to_offset(UtcOffset::UTC);
   Ok(format!(
      "{:04}-{:02}-{:02} {:02}:{:02}:{:02} UTC",
      value.year(),
      u8::from(value.month()),
      value.day(),
      value.hour(),
      value.minute(),
      value.second(),
   ))
}

pub fn datetime_attribute(value: &OffsetDateTime, _: &dyn Values) -> askama::Result<String> {
   value
      .to_offset(UtcOffset::UTC)
      .format(&Rfc3339)
      .map_err(askama::Error::custom)
}

#[cfg(test)]
mod tests {
   use super::*;

   #[test]
   fn formats_in_utc_without_subseconds() {
      let value = OffsetDateTime::from_unix_timestamp(1_234_567_890)
         .unwrap()
         .to_offset(UtcOffset::from_hms(5, 30, 0).unwrap());

      assert_eq!(
         ui_datetime(&value, askama::NO_VALUES).unwrap(),
         "2009-02-13 23:31:30 UTC"
      );
      assert_eq!(
         datetime_attribute(&value, askama::NO_VALUES).unwrap(),
         "2009-02-13T23:31:30Z"
      );
   }
}
