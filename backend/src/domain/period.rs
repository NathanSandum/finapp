use chrono::NaiveDate;

use crate::error::AppError;

#[derive(Debug, Clone, serde::Serialize)]
pub struct Period {
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
    pub label: String,
}

impl Period {
    pub fn parse(year: &str, period: &str) -> Result<Period, AppError> {
        let year_num: i32 = year.parse().map_err(|_| {
            AppError::Validation(vec![crate::error::FieldError {
                field: "year".into(),
                message: format!("invalid year: {year}"),
            }])
        })?;

        match period {
            "Jan" => Self::month(year_num, 1),
            "Feb" => Self::month(year_num, 2),
            "Mar" => Self::month(year_num, 3),
            "Apr" => Self::month(year_num, 4),
            "May" => Self::month(year_num, 5),
            "Jun" => Self::month(year_num, 6),
            "Jul" => Self::month(year_num, 7),
            "Aug" => Self::month(year_num, 8),
            "Sep" => Self::month(year_num, 9),
            "Oct" => Self::month(year_num, 10),
            "Nov" => Self::month(year_num, 11),
            "Dec" => Self::month(year_num, 12),
            "Q1" => Self::quarter(year_num, 1),
            "Q2" => Self::quarter(year_num, 2),
            "Q3" => Self::quarter(year_num, 3),
            "Q4" => Self::quarter(year_num, 4),
            "Year" => Ok(Period {
                start_date: format!("{year_num}-01-01"),
                end_date: format!("{year_num}-12-31"),
                label: "Year".into(),
            }),
            _ => Err(AppError::Validation(vec![crate::error::FieldError {
                field: "period".into(),
                message: format!("invalid period: {period}. Expected Jan..Dec, Q1..Q4, or Year"),
            }])),
        }
    }

    fn month(year: i32, month: u32) -> Result<Period, AppError> {
        let label = match month {
            1 => "Jan", 2 => "Feb", 3 => "Mar", 4 => "Apr",
            5 => "May", 6 => "Jun", 7 => "Jul", 8 => "Aug",
            9 => "Sep", 10 => "Oct", 11 => "Nov", 12 => "Dec",
            _ => unreachable!(),
        };
        let start_date = NaiveDate::from_ymd_opt(year, month, 1).unwrap();
        let end_date = Self::last_day_of_month(year, month);
        Ok(Period {
            start_date,
            end_date,
            label: label.into(),
        })
    }

    fn quarter(year: i32, quarter: u32) -> Result<Period, AppError> {
        let first_month = (quarter - 1) * 3 + 1;
        let last_month = first_month + 2;
        let start_date = NaiveDate::from_ymd_opt(year, first_month, 1).unwrap();
        let end_date = Self::last_day_of_month(year, last_month);
        Ok(Period {
            start_date,
            end_date,
            label: format!("Q{quarter}"),
        })
    }

    fn last_day_of_month(year: i32, month: u32) -> NaiveDate {
        let day = match month {
            1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
            4 | 6 | 9 | 11 => 30,
            2 => {
                if year % 4 == 0 && (year % 100 != 0 || year % 400 == 0) {
                    29
                } else {
                    28
                }
            }
            _ => unreachable!(),
        };
        NaiveDate::from_ymd_opt(year, month, day).unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_january() {
        let period = Period::parse("2026", "Jan").unwrap();
        assert_eq!(period.start_date, "2026-01-01");
        assert_eq!(period.end_date, "2026-01-31");
        assert_eq!(period.label, "Jan");
    }

    #[test]
    fn parse_february_non_leap() {
        let period = Period::parse("2026", "Feb").unwrap();
        assert_eq!(period.end_date, "2026-02-28");
    }

    #[test]
    fn parse_february_leap() {
        let period = Period::parse("2024", "Feb").unwrap();
        assert_eq!(period.end_date, "2024-02-29");
    }

    #[test]
    fn parse_december() {
        let period = Period::parse("2026", "Dec").unwrap();
        assert_eq!(period.start_date, "2026-12-01");
        assert_eq!(period.end_date, "2026-12-31");
    }

    #[test]
    fn parse_q1() {
        let period = Period::parse("2026", "Q1").unwrap();
        assert_eq!(period.start_date, "2026-01-01");
        assert_eq!(period.end_date, "2026-03-31");
        assert_eq!(period.label, "Q1");
    }

    #[test]
    fn parse_q4() {
        let period = Period::parse("2026", "Q4").unwrap();
        assert_eq!(period.start_date, "2026-10-01");
        assert_eq!(period.end_date, "2026-12-31");
    }

    #[test]
    fn parse_year() {
        let period = Period::parse("2026", "Year").unwrap();
        assert_eq!(period.start_date, "2026-01-01");
        assert_eq!(period.end_date, "2026-12-31");
        assert_eq!(period.label, "Year");
    }

    #[test]
    fn parse_invalid_period() {
        assert!(Period::parse("2026", "Foo").is_err());
    }

    #[test]
    fn parse_invalid_year() {
        assert!(Period::parse("abc", "Jan").is_err());
    }
}
