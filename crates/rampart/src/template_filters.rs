#![expect(
   clippy::inline_always,
   clippy::missing_trait_methods,
   clippy::unnecessary_wraps,
   clippy::unused_self,
   reason = "askama::filter_fn expands into a trait impl whose shape we don't control"
)]

use askama::Values;
use time::{
   OffsetDateTime,
   UtcOffset,
   format_description::well_known::Rfc3339,
};

#[askama::filter_fn]
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

#[askama::filter_fn]
pub fn datetime_attribute(value: &OffsetDateTime, _: &dyn Values) -> askama::Result<String> {
   value
      .to_offset(UtcOffset::UTC)
      .format(&Rfc3339)
      .map_err(askama::Error::custom)
}

#[cfg(test)]
#[expect(
   clippy::inline_modules,
   reason = "small cohesive test submodule kept inline"
)]
mod tests {
   use super::*;

   #[test]
   fn formats_in_utc_without_subseconds() {
      let value = OffsetDateTime::from_unix_timestamp(1_234_567_890)
         .unwrap()
         .to_offset(UtcOffset::from_hms(5, 30, 0).unwrap());

      assert_eq!(
         ui_datetime::default()
            .execute(&value, askama::NO_VALUES)
            .unwrap(),
         "2009-02-13 23:31:30 UTC"
      );
      assert_eq!(
         datetime_attribute::default()
            .execute(&value, askama::NO_VALUES)
            .unwrap(),
         "2009-02-13T23:31:30Z"
      );
   }
}
