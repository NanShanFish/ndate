use chinese_lunisolar_calendar::{chrono::{DateTime, Datelike, Duration, Local, TimeZone, Timelike}, LunarDay, LunarMonth, LunisolarDate, LunisolarYear, SolarYear};
use regex::{Match, Regex};
use lazy_static::lazy_static;
lazy_static! {
    static ref DATE_TIME_PAT: Regex = unsafe {
        Regex::new(r"(?:(\d{2}{1,2})[/-])?(\d{1,2})[/-](\d{1,2})(?: (\d{1,2}):(\d{2}))?")
            .unwrap_unchecked()
    };
    static ref DELTA_TIME_PAT: Regex = unsafe {
        Regex::new(r"([+-]?\d+)(?: (\d{1,2}):(\d{2}))?").unwrap_unchecked()
    };
}

fn hour_and_min(h_matched: Option<Match<'_>>, m_matched: Option<Match<'_>>) -> (u32, u32) {
    match (h_matched, m_matched) {
        (Some(h_str), Some(m_str)) => {
            let hour: u32 = unsafe { h_str.as_str().parse().unwrap_unchecked() };
            let minute: u32 = unsafe { m_str.as_str().parse().unwrap_unchecked() };
            (hour, minute)
        },
        _ => (0, 0)
    }
}


#[allow(clippy::too_many_arguments)]
fn date_from_ymd_and_hm(
    year: i32, month: u32, day: u32,
    hour: u32, minute: u32,
    is_lunar: bool, next: bool, now: &DateTime<Local>
) -> DateTime<Local> {
    if is_lunar {
        let lunar_year = LunisolarYear::from_solar_year(SolarYear::from_u16(year as u16))
            .expect("Invalid lunar year");
        let is_leap_month = match lunar_year.get_leap_lunar_month() {
            Some(leap_month) => leap_month.to_u8() == month as u8,
            None => false,
        };

        let lunar_date =
            LunisolarDate::from_lunisolar_year_lunar_month_day(
                lunar_year,
                LunarMonth::from_u8_with_leap(month as u8, is_leap_month)
                .expect("Invalid lunar month"),
                LunarDay::from_u8(day as u8).expect("Invalid lunar day"))
            .expect("Invalid lunar date combination");
        let native_datetime = lunar_date.to_naive_date().and_hms_opt(hour, minute, 0)
            .expect("Invalid hour or minute value");
        let res = Local.from_local_datetime(&native_datetime)
            .single().unwrap();
        if next && res < *now {
            date_from_ymd_and_hm(year+1, month, day, hour, minute, is_lunar, next, now)
        } else {
            res
        }
    } else {
        let res = Local.with_ymd_and_hms(year, month, day, hour, minute, 0)
            .single().expect("Invalid or out-of-range datetime provided" );
        if next && res < *now {
            res.with_year(year + 1).expect("Year adjustment resulted in invalid datetime")
        } else {
            res
        }
    }
}

pub fn parse_datetime(s: String, is_lunar: bool, next: bool) -> DateTime<Local> {
    let begin_of_today= Local::now()
        .with_hour(0)
        .and_then(|dt| dt.with_minute(0))
        .and_then(|dt| dt.with_second(0))
        .and_then(|dt| dt.with_nanosecond(0))
        .unwrap();

    if let Some(caps) = DATE_TIME_PAT.captures(&s) {
        // \d{1,2} 转 int 不会失败
        let month: u32 = unsafe { caps[2].parse().unwrap_unchecked() };
        let day: u32 = unsafe { caps[3].parse().unwrap_unchecked() };
        let need_next;
        let year = match caps.get(1) {
            // \d{2}{2} 转 int 不会失败
            Some(matched) => {
                let res = unsafe { matched.as_str().parse().unwrap_unchecked() };
                need_next = false;
                if res < 100 {
                    2000 + res
                } else {
                    res
                }
            },
            None => {
                need_next = next;
                begin_of_today.year()
            }
        };
        let (hour, minute) = hour_and_min(caps.get(4), caps.get(5));
        date_from_ymd_and_hm(year, month, day, hour, minute, is_lunar, need_next, &begin_of_today)
    } else if let Some(caps) = DELTA_TIME_PAT.captures(&s) {
        let day_delta: i64 = unsafe { caps[1].parse().unwrap_unchecked() };
        let mut result = begin_of_today;
        let duration = Duration::days(day_delta);
        result = result.checked_add_signed(duration)
            .expect("Duration calculation overflowed");

        let (hour, minute) = hour_and_min(caps.get(2), caps.get(3));
        result.with_hour(hour)
            .and_then(|dt| dt.with_minute(minute))
            .expect("Invalid time component specified")
    } else {
        panic!("Input format not recognized");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_date() {
        let time = unsafe {
            Local.with_ymd_and_hms(2025, 2, 1, 0, 0, 0)
                .single().unwrap_unchecked()
        };
        let duration = Duration::days(1);
        let tmp = time.checked_sub_signed(duration).unwrap();
        let s = format!("{}-{}", tmp.month(), tmp.day());

        assert_eq!(
            unsafe{
                Local.with_ymd_and_hms(2026, 1, 31, 0, 0, 0)
                    .single().unwrap_unchecked()
            },
            parse_datetime(s.clone(), false, true)
        );
        assert_eq!(
            unsafe{
                Local.with_ymd_and_hms(2025, 1, 31, 0, 0, 0)
                    .single().unwrap_unchecked()
            },
            parse_datetime(s, false, false)
        );
    }

    #[test]
    fn year_date_time() {
        let s = String::from("2025-10-25 3:03");

        assert_eq!(
            unsafe{
                Local.with_ymd_and_hms(2025, 10, 25, 3, 3, 0)
                    .single().unwrap_unchecked()
            },
            parse_datetime(s, false, true)
        )
    }

    #[test]
    fn date_time() {
        let s = String::from("10-25 3:03");
        let now = Local::now();

        assert_eq!(
            unsafe{
                Local.with_ymd_and_hms(now.year(), 10, 25, 3, 3, 0)
                    .single().unwrap_unchecked()
            },
            parse_datetime(s, false, true)
        )
    }

    #[test]
    fn date_delta() {
        let s = String::from("-1");
        let time = Local::now()
            .with_hour(0)
            .and_then(|dt| dt.with_minute(0))
            .and_then(|dt| dt.with_second(0))
            .and_then(|dt| dt.with_nanosecond(0))
            .unwrap()
            .checked_sub_signed(Duration::days(1)).unwrap();

        assert_eq!(
            time,
            parse_datetime(s, false, true)
        )
    }

    #[test]
    fn luar_date() {
        let s = String::from("2024-2-30");

        assert_eq!(
            unsafe{
                Local.with_ymd_and_hms(2024, 4, 8, 0, 0, 0)
                    .single().unwrap_unchecked()
            },
            parse_datetime(s, true, true)
        )
    }
}
