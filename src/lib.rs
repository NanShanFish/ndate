use chinese_lunisolar_calendar::chrono::{DateTime, Datelike, Duration, TimeZone, Timelike, Utc};
use regex::Regex;
use lazy_static::lazy_static;
lazy_static! {
    static ref DATE_TIME_PAT: Regex =
        Regex::new(r"(?:(\d{2}{1,2})[/-])?(\d{1,2})[/-](\d{1,2})(?: (\d{1,2}):(\d{2}))?").unwrap();
    static ref DELTA_TIME_PAT: Regex =
        Regex::new(r"([+-]\d+)(?: (\d{1,2}):(\d{2}))?").unwrap();
}

fn get_time_now() -> DateTime<Utc> {
    Utc::now()
    .checked_add_signed(Duration::hours(8)).expect("转换时区失败")
}

pub fn parse_datetime(s: String) -> DateTime<Utc> {
    let now = get_time_now()
        .with_second(0)
        .and_then(|dt| dt.with_nanosecond(0))
        .unwrap();

    if let Some(caps) = DATE_TIME_PAT.captures(&s) {
        // \d{1,2} 转 int 不会失败
        let month: u32 = unsafe { caps[2].parse().unwrap_unchecked() };
        let day: u32 = unsafe { caps[3].parse().unwrap_unchecked() };
        let year = match caps.get(1) {
            // \d{2}{2} 转 int 不会失败
            Some(matched) => {
                let res = unsafe { matched.as_str().parse().unwrap_unchecked() };
                if res < 100 {
                    2000 + res
                } else {
                    res
                }
            },
            None => {
                if now.month() > month || now.month() == month && now.day() >= day {
                    now.year()
                } else {
                    now.year() + 1
                }
            }
        };
        match (caps.get(4), caps.get(5)) {
            (Some(h_str), Some(m_str)) => {
                let hour: u32 = unsafe { h_str.as_str().parse().unwrap_unchecked() };
                let minute: u32 = unsafe { m_str.as_str().parse().unwrap_unchecked() };
                Utc.with_ymd_and_hms(year, month, day, hour, minute, 0)
                    .single().expect("提供的时间或日期无效" )
            },
            _ => Utc.with_ymd_and_hms(year, month, day, 0, 0, 0)
                .single().expect("提供的时间或日期无效" )
        }
    } else if let Some(caps) = DELTA_TIME_PAT.captures(&s) {
        let day_delta: i64 = unsafe { caps[1].parse().unwrap_unchecked() };
        let mut result = now;
        let duration = Duration::days(day_delta);
        result = result.checked_add_signed(duration)
            .expect("时间计算溢出");

        let (hour, minute) = match (caps.get(2), caps.get(3)) {
            (Some(h_str), Some(m_str)) => {
                let hour: u32 = unsafe { h_str.as_str().parse().unwrap_unchecked() };
                let minute: u32 = unsafe { m_str.as_str().parse().unwrap_unchecked() };
                (hour, minute)
            },
            _ => (0, 0),
        };
        println!("{}",result);
        result = result.with_hour(hour)
            .and_then(|dt| dt.with_minute(minute))
            .expect("无效的时间");
        result
    } else {
        panic!("无效的格式");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_date() {
        let time = unsafe { Utc.with_ymd_and_hms(2025, 2, 1, 0, 0, 0).single().unwrap_unchecked() };
        let duration = Duration::days(1);
        let tmp = time.checked_sub_signed(duration).unwrap();
        let s = format!("{}-{}", tmp.month(), tmp.day());

        assert_eq!(
            unsafe{ Utc.with_ymd_and_hms(2025, 1, 31, 0, 0, 0).single().unwrap_unchecked() },
            parse_datetime(s)
        )
    }

    #[test]
    fn year_date_time() {
        let s = String::from("2025-10-25 3:03");

        assert_eq!(
            unsafe{ Utc.with_ymd_and_hms(2025, 10, 25, 3, 3, 0).single().unwrap_unchecked() },
            parse_datetime(s)
        )
    }

    #[test]
    fn date_time() {
        let s = String::from("10-25 3:03");
        let now = Utc::now();

        assert_eq!(
            unsafe{ Utc.with_ymd_and_hms(now.year(), 10, 25, 3, 3, 0).single().unwrap_unchecked() },
            parse_datetime(s)
        )
    }

    #[test]
    fn date_delta() {
        let s = String::from("-1");
        let time = get_time_now()
            .with_hour(0)
            .and_then(|dt| dt.with_minute(0))
            .and_then(|dt| dt.with_second(0))
            .and_then(|dt| dt.with_nanosecond(0))
            .unwrap()
            .checked_sub_signed(Duration::days(1)).unwrap();

        assert_eq!(
            time,
            parse_datetime(s)
        )
    }
}
