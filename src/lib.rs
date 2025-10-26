use chinese_lunisolar_calendar::{chrono::{DateTime, Datelike, Duration, Local, TimeZone, Timelike}, LunarDay, LunarMonth, LunisolarDate, LunisolarYear, SolarYear};
use regex::{Match, Regex};
use lazy_static::lazy_static;
lazy_static! {
    static ref DATE_TIME_PAT: Regex = unsafe {
        Regex::new(r"(?:\b(\d{2}{1,2})[/-])?\b(\d{1,2})[/-](\d{1,2})(?: (\d{1,2}):(\d{2}))?")
            .unwrap_unchecked()
    };
    static ref DELTA_TIME_PAT: Regex = unsafe {
        Regex::new(r"^([+-]?\d+)(?: (\d{1,2}):(\d{2}))?$").unwrap_unchecked()
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
    is_lunar: &bool, next: &bool, now: &DateTime<Local>
) -> DateTime<Local> {
    if *is_lunar {
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
        if *next && res < *now {
            date_from_ymd_and_hm(year+1, month, day, hour, minute, is_lunar, next, now)
        } else {
            res
        }
    } else {
        let res = Local.with_ymd_and_hms(year, month, day, hour, minute, 0)
            .single().expect("Invalid or out-of-range datetime provided" );
        if *next && res < *now {
            res.with_year(year + 1).expect("Year adjustment resulted in invalid datetime")
        } else {
            res
        }
    }
}

fn datetime_to_string(datetime: DateTime<Local>, fstr: &Option<String>) -> String {
    if let Some(fstr) = fstr {
        datetime.format(fstr).to_string()
    } else {
        datetime.format("%Y-%m-%d %H:%M").to_string()
    }
}

pub fn parse_datetime(s: String, is_lunar: bool, next: bool, fstr: &Option<String>) -> String  {
    let begin_of_today= Local::now()
        .with_hour(0)
        .and_then(|dt| dt.with_minute(0))
        .and_then(|dt| dt.with_second(0))
        .and_then(|dt| dt.with_nanosecond(0))
        .unwrap();

    if let Some(caps) = DELTA_TIME_PAT.captures(&s) {
        let day_delta: i64 = unsafe { caps[1].parse().unwrap_unchecked() };
        let mut result = begin_of_today;
        let duration = Duration::days(day_delta);
        result = result.checked_add_signed(duration)
            .expect("Duration calculation overflowed");

        let (hour, minute) = hour_and_min(caps.get(2), caps.get(3));
        datetime_to_string(
            result.with_hour(hour)
            .and_then(|dt| dt.with_minute(minute))
            .expect("Invalid time component specified"),
            fstr
        )
    } else {
        let result = DATE_TIME_PAT.replace_all(&s, | caps: &regex::Captures |{
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
            datetime_to_string(
                date_from_ymd_and_hm(
                    year, month, day, hour, minute,
                    &is_lunar, &need_next, &begin_of_today
                ),
                fstr
            )
        });
        result.to_string()
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
            String::from("2026-01-31 00:00"),
            parse_datetime(s.clone(), false, true, &None)
        );
        assert_eq!(
            String::from("2025-01-31 00:00"),
            parse_datetime(s, false, false, &None)
        );
    }

    #[test]
    fn year_date_time() {
        let s = String::from("2025-10-25 03:03");

        assert_eq!(
            s,
            parse_datetime(s.clone(), false, true, &None)
        )
    }

    #[test]
    fn date_time() {
        let s = String::from("10-25 3:03");
        let now = Local::now();

        assert_eq!(
            format!("{}-10-25 03:03", now.year()),
            parse_datetime(s, false, true, &None)
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
            format!("{}-{}-{} 00:00", time.year(), time.month(), time.day()),
            parse_datetime(s, false, true, &None)
        )
    }

    #[test]
    fn luar_date() {
        let s = String::from("2024-2-30");

        assert_eq!(
            String::from("2024-04-08 00:00"),
            parse_datetime(s, true, true, &None)
        )
    }
}
