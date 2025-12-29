//! Conversion functions between q types and Rust types.

use crate::error::Error;
use crate::qconsts::qnull_base;
use crate::qnull_inf::{qinf, qninf, qnull};
use crate::types::Result;
use chrono::prelude::*;
use chrono::Duration;

/// kdb+ offset constants
pub const ONE_DAY_NANOS: i64 = 86400000000000;
pub const ONE_DAY_MILLIS: i64 = 86400000;
pub const KDB_MONTH_OFFSET: i32 = 360;
pub const KDB_DAY_OFFSET: i32 = 10957;
pub const KDB_TIMESTAMP_OFFSET: i64 = 946684800000000000;

/// Convert q timestamp (elapsed time in nanoseconds since `2000.01.01D00:00:00`) into `DateTime<Utc>`.
pub fn q_timestamp_to_datetime(nanos: i64) -> DateTime<Utc> {
    // q          |----------------------------------------|
    // Rust  |----------------------------------------------------|

    // Add duration to avoid overflow
    Utc.timestamp_nanos(nanos) + Duration::nanoseconds(KDB_TIMESTAMP_OFFSET)
}

/// Convert q month (elapsed time in months since `2000.01.01`) into `Date<Utc>`.
pub fn q_month_to_date(months: i32) -> NaiveDate {
    // q     |------------------------------------------------------|
    // Rust        |-----------------------------------------|

    if months == qnull_base::I {
        qnull::MONTH
    } else if months <= -3171072 {
        // Consider pulling month value from q, not only reverse Rust->q.
        // Convert Date::signed_duration_since(chrono::MIN_DATE, Utc.ymd(2000, 1,1)).num_days()) into months
        //  with 1461 as 4 years, 36525 as 100 years and 146097 as 400 years
        *qninf::MONTH
    } else if months >= 3121728 {
        // Consider pulling month value from q, not only reverse Rust->q.
        // Convert Date::signed_duration_since(chrono::MAX_DATE - Duration::days(30), Utc.ymd(2000, 1,1)).num_days()) into months
        //  with 1461 as 4 years, 36525 as 100 years and 146097 as 400 years
        *qinf::MONTH
    } else {
        NaiveDate::from_ymd_opt(2000 + months / 12, 1 + (months % 12) as u32, 1).unwrap()
    }
}

/// Convert q month (elapsed time in days since `2000.01.01`) into `Date<Utc>`.
pub fn q_date_to_date(days: i32) -> Result<NaiveDate> {
    // q     |------------------------------------------------------|
    // Rust        |-----------------------------------------|

    if days == qnull_base::I {
        Ok(qnull::DATE)
    } else if days <= -96476615 {
        // Consider pulling date value from q, not only reverse Rust->q.
        // Date::signed_duration_since(chrono::MIN_DATE, Utc.ymd(2000, 1,1)).num_days())
        Ok(*qninf::DATE)
    } else if days >= 95015644 {
        // Consider pulling date value from q, not only reverse Rust->q.
        // Date::signed_duration_since(chrono::MAX_DATE, Utc.ymd(2000, 1,1)).num_days())
        Ok(qinf::DATE)
    } else {
        Ok((NaiveDate::from_ymd_opt(2000, 1, 1)
            .ok_or_else(|| Error::InvalidDateTime)?
            .and_hms_opt(0, 0, 0)
            .ok_or_else(|| Error::InvalidDateTime)?
            .and_local_timezone(Utc)
            .unwrap()
            + Duration::days(days as i64))
        .date_naive())
    }
}

/// Convert q datetime (elapsed time in days with glanularity of milliseconds since `2000.01.01T00:00:00`) into `DateTime<Utc>`.
pub fn q_datetime_to_datetime(days: f64) -> DateTime<Utc> {
    // q     |------------------------------------------------------|
    // Rust        |-----------------------------------------|

    if days.is_nan() {
        qnull::DATETIME
    } else if days <= -96476615 as f64 {
        // Consider pulling datetime value from q, not only reverse Rust->q.
        // DateTime::signed_duration_since(chrono::MIN_DATETIME, Utc.ymd(2000,1,1).and_hms_nano(0, 0, 0, 0)).num_days())
        *qninf::DATETIME
    } else if days >= 95015644 as f64 {
        // Consider pulling datetime value from q, not only reverse Rust->q.
        // DateTime::signed_duration_since(chrono::MAX_DATETIME, Utc.ymd(2000,1,1).and_hms_nano(0, 0, 0, 0)).num_days())
        *qinf::DATETIME
    } else {
        Utc.timestamp_millis_opt((ONE_DAY_MILLIS as f64 * (days + KDB_DAY_OFFSET as f64)) as i64)
            .unwrap()
    }
}

/// Convert q timespan into `Duration`.
pub fn q_timespan_to_duration(nanos: i64) -> Duration {
    Duration::nanoseconds(nanos)
}

/// Convert q minute into `Duration`.
pub fn q_minute_to_duration(minutes: i32) -> Duration {
    Duration::minutes(minutes as i64)
}

/// Convert q second into `Duration`.
pub fn q_second_to_duration(seconds: i32) -> Duration {
    Duration::seconds(seconds as i64)
}

/// Convert q time into `Duration`.
pub fn q_time_to_duration(millis: i32) -> Duration {
    Duration::milliseconds(millis as i64)
}
