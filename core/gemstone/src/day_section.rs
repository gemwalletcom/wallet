use chrono::{Duration, NaiveDate};

#[derive(Debug, Clone, Copy, PartialEq, Eq, uniffi::Record)]
pub struct GemDay {
    pub year: i32,
    pub month: u32,
    pub day: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, uniffi::Record)]
pub struct GemDayBoundaries {
    pub today: GemDay,
    pub yesterday: GemDay,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, uniffi::Enum)]
pub enum GemDayLabel {
    Today,
    Yesterday,
    Date,
}

#[uniffi::export]
impl GemDayBoundaries {
    pub fn label(&self, day: GemDay) -> GemDayLabel {
        match day {
            day if day == self.today => GemDayLabel::Today,
            day if day == self.yesterday => GemDayLabel::Yesterday,
            _ => GemDayLabel::Date,
        }
    }
}

#[uniffi::export]
impl GemDay {
    pub fn boundaries(&self) -> GemDayBoundaries {
        let yesterday = self.date().and_then(|date| date.checked_sub_signed(Duration::days(1))).map(GemDay::from);
        GemDayBoundaries {
            today: *self,
            yesterday: yesterday.unwrap_or(*self),
        }
    }
}

impl GemDay {
    fn date(&self) -> Option<NaiveDate> {
        NaiveDate::from_ymd_opt(self.year, self.month, self.day)
    }
}

impl From<NaiveDate> for GemDay {
    fn from(date: NaiveDate) -> Self {
        use chrono::Datelike;
        Self {
            year: date.year(),
            month: date.month(),
            day: date.day(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_yesterday_steps_back_over_a_month_and_a_year_boundary() {
        assert_eq!(GemDay { year: 2026, month: 3, day: 1 }.boundaries().yesterday, GemDay { year: 2026, month: 2, day: 28 });
        assert_eq!(GemDay { year: 2026, month: 1, day: 1 }.boundaries().yesterday, GemDay { year: 2025, month: 12, day: 31 });
        assert_eq!(GemDay { year: 2024, month: 3, day: 1 }.boundaries().yesterday, GemDay { year: 2024, month: 2, day: 29 });
    }

    #[test]
    fn test_a_day_is_named_today_or_yesterday_before_it_is_dated() {
        let boundaries = GemDay { year: 2026, month: 3, day: 1 }.boundaries();

        assert_eq!(boundaries.label(GemDay { year: 2026, month: 3, day: 1 }), GemDayLabel::Today);
        assert_eq!(boundaries.label(GemDay { year: 2026, month: 2, day: 28 }), GemDayLabel::Yesterday);
        assert_eq!(boundaries.label(GemDay { year: 2026, month: 2, day: 27 }), GemDayLabel::Date);
        assert_eq!(
            boundaries.label(GemDay { year: 2026, month: 3, day: 2 }),
            GemDayLabel::Date,
            "a future day is dated, not named"
        );
    }

    #[test]
    fn test_an_impossible_today_has_no_separate_yesterday() {
        let boundaries = GemDay { year: 2026, month: 2, day: 30 }.boundaries();

        assert_eq!(boundaries.today, boundaries.yesterday);
    }
}
