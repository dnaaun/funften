use chrono::{NaiveDate, NaiveDateTime};

/// Supposed to parse (forgivingly) input in the format of: YYYY-MM-DDThh:mm
/// Where T can also be a space character. Should also be forgiving of missing
/// months, days, hours, minutes, and seconds.
/// https://developer.mozilla.org/en-US/docs/Web/HTML/Element/input/datetime-local
/// This was written mostly by GPT-4 and copilot.
pub fn parse_input_datetime<P: AsRef<str>>(s: P) -> Option<NaiveDateTime> {
    let s = s.as_ref();
    let mut parts = s.split(|c| c == 'T' || c == ' ');

    let date = parts.next()?;
    let time = match parts.next() {
        Some(time) if !time.is_empty() => time,
        _ => "0:0",
    };

    let mut date_parts = date.split('-');
    let year = date_parts.next()?.parse::<i32>().ok()?;
    let month = date_parts.next().unwrap_or("1").parse::<u32>().ok()?;
    let day = date_parts.next().unwrap_or("1").parse::<u32>().ok()?;

    let mut time_parts = time.split(':');
    let hour = time_parts.next().unwrap_or("0").parse::<u32>().ok()?;
    let minute = {
        let minute_str = time_parts.next().unwrap_or("0");
        if minute_str.is_empty() {
            0
        } else {
            minute_str.parse::<u32>().ok()?
        }
    };
    let second = time_parts.next().unwrap_or("0").parse::<u32>().ok()?;

    NaiveDate::from_ymd_opt(year, month, day)?.and_hms_opt(hour, minute, second)
}

#[cfg(test)]
mod tests {
    use chrono::NaiveDateTime;

    use crate::utils::date::parse_input_datetime::parse_input_datetime;

    #[test]
    fn test_fully_formatted_input() {
        let input = "2023-05-19T13:45";
        let expected_output =
            NaiveDateTime::parse_from_str("2023-05-19T13:45:00", "%Y-%m-%dT%H:%M:%S");
        assert_eq!(parse_input_datetime(input), expected_output.ok());
    }

    #[test]
    fn test_input_with_space_separator() {
        let input = "2023-05-19 13:45";
        let expected_output =
            NaiveDateTime::parse_from_str("2023-05-19T13:45:00", "%Y-%m-%dT%H:%M:%S");
        assert_eq!(parse_input_datetime(input), expected_output.ok());
    }

    #[test]
    fn test_input_missing_seconds() {
        let input = "2023-05-19T13:45";
        let expected_output =
            NaiveDateTime::parse_from_str("2023-05-19T13:45:00", "%Y-%m-%dT%H:%M:%S");
        assert_eq!(parse_input_datetime(input), expected_output.ok());
    }

    #[test]
    fn test_missing_hours_minutes_seconds() {
        let input = "2023-05-19";
        let expected_output =
            NaiveDateTime::parse_from_str("2023-05-19T00:00:00", "%Y-%m-%dT%H:%M:%S");
        assert_eq!(parse_input_datetime(input), expected_output.ok(),);
    }

    #[test]
    fn test_empty_input() {
        let input = "";
        assert_eq!(parse_input_datetime(input), None);
    }
    #[test]
    fn test_input_missing_day() {
        let input = "2023-05T13:45";
        let expected_output =
            NaiveDateTime::parse_from_str("2023-05-01T13:45:00", "%Y-%m-%dT%H:%M:%S");
        assert_eq!(parse_input_datetime(input), expected_output.ok());
    }

    #[test]
    fn test_input_missing_month_and_day() {
        let input = "2023T13:45";
        let expected_output =
            NaiveDateTime::parse_from_str("2023-01-01T13:45:00", "%Y-%m-%dT%H:%M:%S");
        assert_eq!(parse_input_datetime(input), expected_output.ok());
    }

    #[test]
    fn test_input_missing_hour() {
        let input = "2023-05-19T";
        let expected_output =
            NaiveDateTime::parse_from_str("2023-05-19T00:00:00", "%Y-%m-%dT%H:%M:%S");
        assert_eq!(parse_input_datetime(input), expected_output.ok());
    }

    #[test]
    fn test_input_missing_minute() {
        let input = "2023-05-19T13:";
        let expected_output =
            NaiveDateTime::parse_from_str("2023-05-19T13:00:00", "%Y-%m-%dT%H:%M:%S");
        assert_eq!(parse_input_datetime(input), expected_output.ok());
    }
}
