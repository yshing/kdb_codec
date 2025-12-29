//! Q null, infinity and negative infinity constants.

use crate::qconsts::{qinf_base, qninf_base, qnull_base};

pub mod qnull {
    //! This module provides a list of q null values set on Rust process and used for IPC. The motivation
    //!  to contain them in a module is to tie them up as related items rather than scattered values.
    //!  Hence user should use these indicators with `qnull::` prefix, e.g., `qnull::FLOAT`.

    use super::qnull_base;
    use chrono::prelude::*;
    use chrono::Duration;
    use once_cell::sync::Lazy;

    /// Null value of GUID (`0Ng`).
    /// # Example
    /// ```
    /// use kdb_codec::*;
    ///
    /// fn main() {
    ///     let q_guid_null = K::new_guid(qnull::GUID);
    ///     assert_eq!(
    ///         format!("{}", q_guid_null),
    ///         String::from("00000000-0000-0000-0000-000000000000")
    ///     );
    /// }
    /// ```
    pub const GUID: [u8; 16] = [0_u8; 16];

    /// Null value of short (`0Nh`).
    /// # Example
    /// ```
    /// use kdb_codec::*;
    ///
    /// fn main() {
    ///     let q_short_null = K::new_short(qnull::SHORT);
    ///     assert_eq!(format!("{}", q_short_null), String::from("0Nh"));
    /// }
    /// ```
    pub const SHORT: i16 = qnull_base::H;

    /// Null value of int (`0Ni`).
    /// # Example
    /// ```
    /// use kdb_codec::*;
    ///
    /// fn main() {
    ///     let q_int_null = K::new_int(qnull::INT);
    ///     assert_eq!(format!("{}", q_int_null), String::from("0Ni"));
    /// }
    /// ```
    pub const INT: i32 = qnull_base::I;

    /// Null value of long (`0N`).
    /// # Example
    /// ```
    /// use kdb_codec::*;
    ///
    /// fn main() {
    ///     let q_long_null = K::new_long(qnull::LONG);
    ///     assert_eq!(format!("{}", q_long_null), String::from("0N"));
    /// }
    /// ```
    pub const LONG: i64 = qnull_base::J;

    /// Null value of real (`0Ne`).
    /// # Example
    /// ```
    /// use kdb_codec::*;
    ///
    /// fn main() {
    ///     let q_real_null = K::new_real(qnull::REAL);
    ///     assert_eq!(format!("{}", q_real_null), String::from("0Ne"));
    /// }
    /// ```
    pub const REAL: f32 = qnull_base::E;

    /// Null value of float (`0n`).
    /// # Example
    /// ```
    /// use kdb_codec::*;
    ///
    /// fn main() {
    ///     let q_float_null = K::new_float(qnull::FLOAT);
    ///     assert_eq!(format!("{}", q_float_null), String::from("0n"));
    /// }
    /// ```
    pub const FLOAT: f64 = qnull_base::F;

    /// Null value of char (`" "`).
    /// # Example
    /// ```
    /// use kdb_codec::*;
    ///
    /// fn main() {
    ///     let q_char_null = K::new_char(qnull::CHAR);
    ///     assert_eq!(format!("{}", q_char_null), String::from("\" \""));
    /// }
    /// ```
    pub const CHAR: char = qnull_base::C;

    /// Null value of symbol (<code>`</code>).
    /// # Example
    /// ```
    /// use kdb_codec::*;
    ///
    /// fn main() {
    ///     let q_symbol_null = K::new_symbol(qnull::SYMBOL);
    ///     assert_eq!(format!("{}", q_symbol_null), String::from("`"));
    /// }
    /// ```
    pub const SYMBOL: String = String::new();

    /// Null value of timestamp (`0Np`).
    /// # Example
    /// ```
    /// use kdb_codec::*;
    ///
    /// fn main() {
    ///     let q_timestamp_null = K::new_timestamp(*qnull::TIMESTAMP);
    ///     assert_eq!(format!("{}", q_timestamp_null), String::from("0Np"));
    /// }
    /// ```
    /// # Note
    /// The range of timestamp in Rust is wider than in q.
    pub const TIMESTAMP: Lazy<DateTime<Utc>> = Lazy::new(|| {
        NaiveDate::from_ymd_opt(1707, 9, 22)
            .unwrap()
            .and_hms_nano_opt(0, 12, 43, 145224192)
            .unwrap()
            .and_local_timezone(Utc)
            .unwrap()
    });

    /// Null value of month (`0Nm`).
    /// # Example
    /// ```
    /// use kdb_codec::*;
    ///
    /// fn main() {
    ///     let q_month_null = K::new_month(qnull::MONTH);
    ///     assert_eq!(format!("{}", q_month_null), String::from("0Nm"));
    /// }
    /// ```
    /// # Note
    /// The range of month in Rust is narrower than in q.
    pub const MONTH: NaiveDate = NaiveDate::MIN;

    /// Null valueo of date (`0Nd`).
    /// # Example
    /// ```
    /// use kdb_codec::*;
    ///
    /// fn main() {
    ///     let q_date_null = K::new_date(qnull::DATE);
    ///     assert_eq!(format!("{}", q_date_null), String::from("0Nd"));
    /// }
    /// ```
    /// # Note
    /// The range of date in Rust is narrower than in q.
    pub const DATE: NaiveDate = NaiveDate::MIN;

    /// Null value of datetime (`0Nz`).
    /// # Example
    /// ```
    /// use kdb_codec::*;
    ///
    /// fn main() {
    ///     let q_datetime_null = K::new_datetime(qnull::DATETIME);
    ///     assert_eq!(format!("{}", q_datetime_null), String::from("0Nz"));
    /// }
    /// ```
    /// # Note
    /// The range of datetime in Rust is narrower than in q.
    pub const DATETIME: DateTime<Utc> = DateTime::<Utc>::MIN_UTC;

    /// Null value of timespan (`0Nn`).
    /// # Example
    /// ```
    /// use kdb_codec::*;
    ///
    /// fn main() {
    ///     let q_timespan_null = K::new_timespan(*qnull::TIMESPAN);
    ///     assert_eq!(format!("{}", q_timespan_null), String::from("0Nn"));
    /// }
    /// ```
    pub const TIMESPAN: Lazy<Duration> = Lazy::new(|| Duration::nanoseconds(qnull_base::J));

    /// Null value of minute (`0Nu`).
    /// # Example
    /// ```
    /// use kdb_codec::*;
    ///
    /// fn main() {
    ///     let q_minute_null = K::new_minute(*qnull::MINUTE);
    ///     assert_eq!(format!("{}", q_minute_null), String::from("0Nu"));
    /// }
    /// ```
    pub const MINUTE: Lazy<Duration> = Lazy::new(|| Duration::minutes(qnull_base::I as i64));

    /// Null value of second (`0Nv`).
    /// # Example
    /// ```
    /// use kdb_codec::*;
    ///
    /// fn main() {
    ///     let q_second_null = K::new_second(*qnull::SECOND);
    ///     assert_eq!(format!("{}", q_second_null), String::from("0Nv"));
    /// }
    /// ```
    pub const SECOND: Lazy<Duration> = Lazy::new(|| Duration::seconds(qnull_base::I as i64));

    /// Null value of time (`0Nt`).
    /// # Example
    /// ```
    /// use kdb_codec::*;
    ///
    /// fn main() {
    ///     let q_time_null = K::new_time(*qnull::TIME);
    ///     assert_eq!(format!("{}", q_time_null), String::from("0Nt"));
    /// }
    /// ```
    pub const TIME: Lazy<Duration> = Lazy::new(|| Duration::milliseconds(qnull_base::I as i64));
}

pub mod qinf {
    //! This module provides a list of q infinite values set on Rust process and used for IPC.
    //!  The motivation to contain them in a module is to tie them up as related items rather
    //!  than scattered values. Hence user should use these indicators with `qnull::` prefix, e.g., `qnull::FLOAT`.

    use super::qinf_base;
    use chrono::prelude::*;
    use chrono::Duration;
    use once_cell::sync::Lazy;

    /// Infinity value of short (`0Wh`).
    /// # Example
    /// ```
    /// use kdb_codec::*;
    ///
    /// fn main() {
    ///     let q_short_inf = K::new_short(qinf::SHORT);
    ///     assert_eq!(format!("{}", q_short_inf), String::from("0Wh"));
    /// }
    /// ```
    pub const SHORT: i16 = qinf_base::H;

    /// Infinity value of int (`0Wi`).
    /// # Example
    /// ```
    /// use kdb_codec::*;
    ///
    /// fn main() {
    ///     let q_int_inf = K::new_int(qinf::INT);
    ///     assert_eq!(format!("{}", q_int_inf), String::from("0Wi"));
    /// }
    /// ```
    pub const INT: i32 = qinf_base::I;

    /// Infinity value of long (`0W`).
    /// # Example
    /// ```
    /// use kdb_codec::*;
    ///
    /// fn main() {
    ///     let q_long = K::new_long(86400000000000);
    ///     assert_eq!(format!("{}", q_long), String::from("86400000000000"));
    /// }
    /// ```
    pub const LONG: i64 = qinf_base::J;

    /// Infinity value of real (`0We`).
    /// # Example
    /// ```
    /// use kdb_codec::*;
    ///
    /// fn main() {
    ///     let q_real_null = K::new_real(qnull::REAL);
    ///     assert_eq!(format!("{}", q_real_null), String::from("0Ne"));
    /// }
    /// ```
    pub const REAL: f32 = qinf_base::E;

    /// Infinity value of float (`0w`).
    /// # Example
    /// ```
    /// use kdb_codec::*;
    ///
    /// fn main() {
    ///     let q_float_inf = K::new_float(qinf::FLOAT);
    ///     assert_eq!(format!("{}", q_float_inf), String::from("0w"));
    /// }
    /// ```
    pub const FLOAT: f64 = qinf_base::F;

    /// Infinity value of timestamp (`0Wp`).
    /// # Example
    /// ```
    /// use kdb_codec::*;
    ///
    /// fn main() {
    ///     let q_timestamp_inf = K::new_timestamp(*qinf::TIMESTAMP);
    ///     assert_eq!(format!("{}", q_timestamp_inf), String::from("0Wp"));
    /// }
    /// ```
    /// # Note
    /// The range of timestamp in Rust is wider than in q.
    pub const TIMESTAMP: Lazy<DateTime<Utc>> = Lazy::new(|| {
        NaiveDate::from_ymd_opt(2292, 4, 10)
            .unwrap()
            .and_hms_nano_opt(23, 47, 16, 854775807)
            .unwrap()
            .and_local_timezone(Utc)
            .unwrap()
    });

    /// Infinity value of month (`0Wm`).
    /// # Example
    /// ```
    /// use kdb_codec::*;
    ///
    /// fn main() {
    ///     let q_month_inf = K::new_month(*qinf::MONTH);
    ///     assert_eq!(format!("{}", q_month_inf), String::from("0Wm"));
    /// }
    /// ```
    /// # Note
    /// The range of month in Rust is narrower than in q.
    pub const MONTH: Lazy<NaiveDate> = Lazy::new(|| NaiveDate::MAX - Duration::days(30));

    /// Infinity valueo of date (`0Wd`).
    /// # Example
    /// ```
    /// use kdb_codec::*;
    ///
    /// fn main() {
    ///     let q_date_inf = K::new_date(qinf::DATE);
    ///     assert_eq!(format!("{}", q_date_inf), String::from("0Wd"));
    /// }
    /// ```
    /// # Note
    /// The range of date in Rust is narrower than in q.
    pub const DATE: NaiveDate = NaiveDate::MAX;

    /// Infinity value of datetime (`0Wz`).
    /// # Example
    /// ```
    /// use kdb_codec::*;
    ///
    /// fn main() {
    ///     let q_datetime_inf = K::new_datetime(*qinf::DATETIME);
    ///     assert_eq!(format!("{}", q_datetime_inf), String::from("0Wz"));
    /// }
    /// ```
    /// # Note
    /// The range of datetime in Rust is narrower than in q.
    pub const DATETIME: Lazy<DateTime<Utc>> =
        Lazy::new(|| DateTime::<Utc>::MAX_UTC - Duration::nanoseconds(999999));

    /// Infinity value of timespan (`0Wn`).
    /// # Example
    /// ```
    /// use kdb_codec::*;
    ///
    /// fn main() {
    ///     let q_timespan_inf = K::new_timespan(*qinf::TIMESPAN);
    ///     assert_eq!(format!("{}", q_timespan_inf), String::from("0Wn"));
    /// }
    /// ```
    pub const TIMESPAN: Lazy<Duration> = Lazy::new(|| Duration::nanoseconds(qinf_base::J));

    /// Infinity value of minute (`0Wu`).
    /// # Example
    /// ```
    /// use kdb_codec::*;
    ///
    /// fn main() {
    ///     let q_minute_inf = K::new_minute(*qinf::MINUTE);
    ///     assert_eq!(format!("{}", q_minute_inf), String::from("0Wu"));
    /// }
    /// ```
    pub const MINUTE: Lazy<Duration> = Lazy::new(|| Duration::minutes(qinf_base::I as i64));

    /// Infinity value of second (`0Wv`).
    /// # Example
    /// ```
    /// use kdb_codec::*;
    ///
    /// fn main() {
    ///     let q_second_inf = K::new_second(*qinf::SECOND);
    ///     assert_eq!(format!("{}", q_second_inf), String::from("0Wv"));
    /// }
    /// ```
    pub const SECOND: Lazy<Duration> = Lazy::new(|| Duration::seconds(qinf_base::I as i64));

    /// Infinity value of time (`0Wt`).
    /// # Example
    /// ```
    /// use kdb_codec::*;
    ///
    /// fn main() {
    ///     let q_time_inf = K::new_time(*qinf::TIME);
    ///     assert_eq!(format!("{}", q_time_inf), String::from("0Wt"));
    /// }
    /// ```
    pub const TIME: Lazy<Duration> = Lazy::new(|| Duration::milliseconds(qinf_base::I as i64));
}

pub mod qninf {
    //! This module provides a list of q negative infinite values set on Rust process and used for IPC.
    //!  The motivation to contain them in a module is to tie them up as related items rather than
    //!  scattered values. Hence user should use these indicators with `qnull::` prefix, e.g., `qnull::FLOAT`.

    use super::qninf_base;
    use chrono::prelude::*;
    use chrono::Duration;
    use once_cell::sync::Lazy;

    /// Infinity value of short (`-0Wh`).
    /// # Example
    /// ```
    /// use kdb_codec::*;
    ///
    /// fn main() {
    ///     let q_short_ninf = K::new_short(qninf::SHORT);
    ///     assert_eq!(format!("{}", q_short_ninf), String::from("-0Wh"));
    /// }
    /// ```
    pub const SHORT: i16 = qninf_base::H;

    /// Infinity value of int (`-0Wi`).
    /// # Example
    /// ```
    /// use kdb_codec::*;
    ///
    /// fn main() {
    ///     let q_int_ninf = K::new_int(qninf::INT);
    ///     assert_eq!(format!("{}", q_int_ninf), String::from("-0Wi"));
    /// }
    /// ```
    pub const INT: i32 = qninf_base::I;

    /// Infinity value of long (-`0W`).
    /// # Example
    /// ```
    /// use kdb_codec::*;
    ///
    /// fn main() {
    ///     let q_long_ninf = K::new_long(qninf::LONG);
    ///     assert_eq!(format!("{}", q_long_ninf), String::from("-0W"));
    /// }
    /// ```
    pub const LONG: i64 = qninf_base::J;

    /// Infinity value of real (`-0We`).
    /// # Example
    /// ```
    /// use kdb_codec::*;
    ///
    /// fn main() {
    ///     let q_real_ninf: K = K::new_real(qninf::REAL);
    ///     assert_eq!(format!("{}", q_real_ninf), String::from("-0We"));
    /// }
    /// ```
    pub const REAL: f32 = qninf_base::E;

    /// Infinity value of float (`-0w`).
    /// # Example
    /// ```
    /// use kdb_codec::*;
    ///
    /// fn main() {
    ///     let q_float_ninf = K::new_float(qninf::FLOAT);
    ///     assert_eq!(format!("{}", q_float_ninf), String::from("-0w"));
    /// }
    /// ```
    pub const FLOAT: f64 = qninf_base::F;

    /// Infinity value of timestamp (`-0Wp`).
    /// # Example
    /// ```
    /// use kdb_codec::*;
    ///
    /// fn main() {
    ///     let q_timestamp_ninf = K::new_timestamp(*qninf::TIMESTAMP);
    ///     assert_eq!(format!("{}", q_timestamp_ninf), String::from("-0Wp"));
    /// }
    /// ```
    /// # Note
    /// The range of timestamp in Rust is wider than in q.
    pub const TIMESTAMP: Lazy<DateTime<Utc>> = Lazy::new(|| {
        NaiveDate::from_ymd_opt(1707, 9, 22)
            .unwrap()
            .and_hms_nano_opt(0, 12, 43, 145224193)
            .unwrap()
            .and_local_timezone(Utc)
            .unwrap()
    });

    /// Infinity value of month (`-0Wm`).
    /// # Example
    /// ```
    /// use kdb_codec::*;
    ///
    /// fn main() {
    ///     let q_month_ninf = K::new_month(*qninf::MONTH);
    ///     assert_eq!(format!("{}", q_month_ninf), String::from("-0Wm"));
    /// }
    /// ```
    /// # Note
    /// The range of month in Rust is narrower than in q.
    pub const MONTH: Lazy<NaiveDate> = Lazy::new(|| NaiveDate::MIN + Duration::days(31));

    /// Infinity valueo of date (`-0Wd`).
    /// # Example
    /// ```
    /// use kdb_codec::*;
    ///
    /// fn main() {
    ///     let q_date_ninf = K::new_date(*qninf::DATE);
    ///     assert_eq!(format!("{}", q_date_ninf), String::from("-0Wd"));
    /// }
    /// ```
    /// # Note
    /// The range of date in Rust is narrower than in q.
    pub const DATE: Lazy<NaiveDate> = Lazy::new(|| NaiveDate::MIN + Duration::days(1));

    /// Infinity value of datetime (`-0Wz`).
    /// # Example
    /// ```
    /// use kdb_codec::*;
    ///
    /// fn main() {
    ///     let q_datetime_ninf = K::new_datetime(*qninf::DATETIME);
    ///     assert_eq!(format!("{}", q_datetime_ninf), String::from("-0Wz"));
    /// }
    /// ```
    /// # Note
    /// The range of datetime in Rust is narrower than in q.
    pub const DATETIME: Lazy<DateTime<Utc>> =
        Lazy::new(|| DateTime::<Utc>::MIN_UTC + Duration::nanoseconds(1000000));

    /// Infinity value of timespan (`-0Wn`).
    /// # Example
    /// ```
    /// use kdb_codec::*;
    ///
    /// fn main() {
    ///     let q_timespan_ninf = K::new_timespan(*qninf::TIMESPAN);
    ///     assert_eq!(format!("{}", q_timespan_ninf), String::from("-0Wn"));
    /// }
    /// ```
    pub const TIMESPAN: Lazy<Duration> = Lazy::new(|| Duration::nanoseconds(qninf_base::J));

    /// Infinity value of minute (`-0Wu`).
    /// # Example
    /// ```
    /// use kdb_codec::*;
    ///
    /// fn main() {
    ///     let q_minute_ninf = K::new_minute(*qninf::MINUTE);
    ///     assert_eq!(format!("{}", q_minute_ninf), String::from("-0Wu"));
    /// }
    /// ```
    pub const MINUTE: Lazy<Duration> = Lazy::new(|| Duration::minutes(qninf_base::I as i64));

    /// Infinity value of second (`-0Wv`).
    /// # Example
    /// ```
    /// use kdb_codec::*;
    ///
    /// fn main() {
    ///     let q_second_ninf = K::new_second(*qninf::SECOND);
    ///     assert_eq!(format!("{}", q_second_ninf), String::from("-0Wv"));
    /// }
    /// ```
    pub const SECOND: Lazy<Duration> = Lazy::new(|| Duration::seconds(qninf_base::I as i64));

    /// Infinity value of time (`-0Wt`).
    /// # Example
    /// ```
    /// use kdb_codec::*;
    ///
    /// fn main() {
    ///     let q_time_ninf = K::new_time(*qninf::TIME);
    ///     assert_eq!(format!("{}", q_time_ninf), String::from("-0Wt"));
    /// }
    /// ```
    pub const TIME: Lazy<Duration> = Lazy::new(|| Duration::milliseconds(qninf_base::I as i64));
}
