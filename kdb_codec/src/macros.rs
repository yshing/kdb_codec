//! Macro module providing q-like syntax for constructing K objects.
//!
//! The `k!` macro allows users to create K objects using a more concise syntax,
//! reducing boilerplate and making kdb+ development in Rust more intuitive.
//!
//! # Examples
//!
//! ```
//! use kdb_codec::*;
//!
//! // Atoms
//! let bool_val = k!(bool: true);
//! let long_val = k!(long: 42);
//! let symbol_val = k!(sym: "hello");
//!
//! // Lists - use vec! for values  
//! let int_list = k!(int: vec![1, 2, 3]);
//! let sym_list = k!(sym: vec!["a", "b", "c"]);
//!
//! // Lists with repetition syntax
//! let large_list = k!(long: vec![42; 3000]);  // 3000 copies of 42
//!
//! // Compound lists
//! let compound = k!([k!(long: 1), k!(float: 2.5)]);
//!
//! // Tables
//! let table = k!(table: {
//!     "col1" => k!(int: vec![1, 2, 3]),
//!     "col2" => k!(sym: vec!["a", "b", "c"])
//! });
//! ```

/// Main macro for creating K objects with simplified syntax.
///
/// ## Atoms
/// - `k!(bool: value)` → boolean
/// - `k!(byte: value)` → byte
/// - `k!(short: value)` → short
/// - `k!(int: value)` → int
/// - `k!(long: value)` → long
/// - `k!(real: value)` → real
/// - `k!(float: value)` → float
/// - `k!(char: value)` → char
/// - `k!(sym: "text")` → symbol
/// - `k!(string: "text")` → string
///
/// ## Temporal Atoms
/// - `k!(timestamp: DateTime<Utc>)` → timestamp
/// - `k!(month: NaiveDate)` → month
/// - `k!(date: NaiveDate)` → date
/// - `k!(datetime: DateTime<Utc>)` → datetime
/// - `k!(timespan: Duration)` → timespan
/// - `k!(minute: Duration)` → minute
/// - `k!(second: Duration)` → second
/// - `k!(time: Duration)` → time
///
/// ## Lists
/// Use `vec![...]` for list values:
/// - `k!(bool: vec![...])` → boolean list
/// - `k!(int: vec![...])` → int list
/// - `k!(sym: vec![...])` → symbol list
/// - `k!(timestamp: vec![...])` → timestamp list
/// - `k!(date: vec![...])` → date list
/// - etc.
///
/// Supports both comma-separated and repetition syntax:
/// - `k!(long: vec![1, 2, 3])` → list with specific values
/// - `k!(long: vec![42; 3000])` → list with 3000 copies of 42
///
/// Add `; @sorted`, `; @unique`, `; @parted`, or `; @grouped` for attributes
///
/// ## Compound Lists
/// - `k!([item1, item2, ...])` → compound list
///
/// ## Dictionaries
/// - `k!(dict: keys => values)` → dictionary
///
/// ## Tables
/// - `k!(table: { "col1" => values1, "col2" => values2 })` → table
///
#[macro_export]
macro_rules! k {
    // ========== Lists (must come BEFORE atoms to match first) ==========

    // Boolean list (use vec! explicitly)
    (bool: $val:expr; @$attr:ident) => {{
        // Hack to distinguish vec from scalar: if it's a Vec, use it directly
        let list_val: Vec<bool> = $val;
        $crate::K::new_bool_list(list_val, k!(@attr $attr))
    }};
    (bool: vec![$($val:expr),* $(,)?]) => {
        $crate::K::new_bool_list(vec![$($val),*], $crate::qattribute::NONE)
    };
    (bool: vec![$($val:expr),* $(,)?]; @$attr:ident) => {
        $crate::K::new_bool_list(vec![$($val),*], k!(@attr $attr))
    };

    // Byte list
    (byte: vec![$val:expr; $count:expr]) => {
        $crate::K::new_byte_list(vec![$val; $count], $crate::qattribute::NONE)
    };
    (byte: vec![$val:expr; $count:expr]; @$attr:ident) => {
        $crate::K::new_byte_list(vec![$val; $count], k!(@attr $attr))
    };
    (byte: vec![$($val:expr),* $(,)?]) => {
        $crate::K::new_byte_list(vec![$($val),*], $crate::qattribute::NONE)
    };
    (byte: vec![$($val:expr),* $(,)?]; @$attr:ident) => {
        $crate::K::new_byte_list(vec![$($val),*], k!(@attr $attr))
    };

    // Short list
    (short: vec![$val:expr; $count:expr]) => {
        $crate::K::new_short_list(vec![$val; $count], $crate::qattribute::NONE)
    };
    (short: vec![$val:expr; $count:expr]; @$attr:ident) => {
        $crate::K::new_short_list(vec![$val; $count], k!(@attr $attr))
    };
    (short: vec![$($val:expr),* $(,)?]) => {
        $crate::K::new_short_list(vec![$($val),*], $crate::qattribute::NONE)
    };
    (short: vec![$($val:expr),* $(,)?]; @$attr:ident) => {
        $crate::K::new_short_list(vec![$($val),*], k!(@attr $attr))
    };

    // Int list
    (int: vec![$val:expr; $count:expr]) => {
        $crate::K::new_int_list(vec![$val; $count], $crate::qattribute::NONE)
    };
    (int: vec![$val:expr; $count:expr]; @$attr:ident) => {
        $crate::K::new_int_list(vec![$val; $count], k!(@attr $attr))
    };
    (int: vec![$($val:expr),* $(,)?]) => {
        $crate::K::new_int_list(vec![$($val),*], $crate::qattribute::NONE)
    };
    (int: vec![$($val:expr),* $(,)?]; @$attr:ident) => {
        $crate::K::new_int_list(vec![$($val),*], k!(@attr $attr))
    };

    // Long list
    (long: vec![$val:expr; $count:expr]) => {
        $crate::K::new_long_list(vec![$val; $count], $crate::qattribute::NONE)
    };
    (long: vec![$val:expr; $count:expr]; @$attr:ident) => {
        $crate::K::new_long_list(vec![$val; $count], k!(@attr $attr))
    };
    (long: vec![$($val:expr),* $(,)?]) => {
        $crate::K::new_long_list(vec![$($val),*], $crate::qattribute::NONE)
    };
    (long: vec![$($val:expr),* $(,)?]; @$attr:ident) => {
        $crate::K::new_long_list(vec![$($val),*], k!(@attr $attr))
    };

    // Real list
    (real: vec![$val:expr; $count:expr]) => {
        $crate::K::new_real_list(vec![$val; $count], $crate::qattribute::NONE)
    };
    (real: vec![$val:expr; $count:expr]; @$attr:ident) => {
        $crate::K::new_real_list(vec![$val; $count], k!(@attr $attr))
    };
    (real: vec![$($val:expr),* $(,)?]) => {
        $crate::K::new_real_list(vec![$($val),*], $crate::qattribute::NONE)
    };
    (real: vec![$($val:expr),* $(,)?]; @$attr:ident) => {
        $crate::K::new_real_list(vec![$($val),*], k!(@attr $attr))
    };

    // Float list
    (float: vec![$val:expr; $count:expr]) => {
        $crate::K::new_float_list(vec![$val; $count], $crate::qattribute::NONE)
    };
    (float: vec![$val:expr; $count:expr]; @$attr:ident) => {
        $crate::K::new_float_list(vec![$val; $count], k!(@attr $attr))
    };
    (float: vec![$($val:expr),* $(,)?]) => {
        $crate::K::new_float_list(vec![$($val),*], $crate::qattribute::NONE)
    };
    (float: vec![$($val:expr),* $(,)?]; @$attr:ident) => {
        $crate::K::new_float_list(vec![$($val),*], k!(@attr $attr))
    };

    // Symbol list
    (sym: vec![$val:expr; $count:expr]) => {{
        let symbols: Vec<String> = vec![$val.to_string(); $count];
        $crate::K::new_symbol_list(symbols, $crate::qattribute::NONE)
    }};
    (sym: vec![$val:expr; $count:expr]; @$attr:ident) => {{
        let symbols: Vec<String> = vec![$val.to_string(); $count];
        $crate::K::new_symbol_list(symbols, k!(@attr $attr))
    }};
    (sym: vec![$($val:expr),* $(,)?]) => {{
        let symbols: Vec<String> = vec![$($val.to_string()),*];
        $crate::K::new_symbol_list(symbols, $crate::qattribute::NONE)
    }};
    (sym: vec![$($val:expr),* $(,)?]; @$attr:ident) => {{
        let symbols: Vec<String> = vec![$($val.to_string()),*];
        $crate::K::new_symbol_list(symbols, k!(@attr $attr))
    }};

    // Timestamp list (chrono::DateTime<Utc>)
    (timestamp: vec![$val:expr; $count:expr]) => {
        $crate::K::new_timestamp_list(vec![$val; $count], $crate::qattribute::NONE)
    };
    (timestamp: vec![$val:expr; $count:expr]; @$attr:ident) => {
        $crate::K::new_timestamp_list(vec![$val; $count], k!(@attr $attr))
    };
    (timestamp: vec![$($val:expr),* $(,)?]) => {
        $crate::K::new_timestamp_list(vec![$($val),*], $crate::qattribute::NONE)
    };
    (timestamp: vec![$($val:expr),* $(,)?]; @$attr:ident) => {
        $crate::K::new_timestamp_list(vec![$($val),*], k!(@attr $attr))
    };

    // Month list (chrono::NaiveDate)
    (month: vec![$val:expr; $count:expr]) => {
        $crate::K::new_month_list(vec![$val; $count], $crate::qattribute::NONE)
    };
    (month: vec![$val:expr; $count:expr]; @$attr:ident) => {
        $crate::K::new_month_list(vec![$val; $count], k!(@attr $attr))
    };
    (month: vec![$($val:expr),* $(,)?]) => {
        $crate::K::new_month_list(vec![$($val),*], $crate::qattribute::NONE)
    };
    (month: vec![$($val:expr),* $(,)?]; @$attr:ident) => {
        $crate::K::new_month_list(vec![$($val),*], k!(@attr $attr))
    };

    // Date list (chrono::NaiveDate)
    (date: vec![$val:expr; $count:expr]) => {
        $crate::K::new_date_list(vec![$val; $count], $crate::qattribute::NONE)
    };
    (date: vec![$val:expr; $count:expr]; @$attr:ident) => {
        $crate::K::new_date_list(vec![$val; $count], k!(@attr $attr))
    };
    (date: vec![$($val:expr),* $(,)?]) => {
        $crate::K::new_date_list(vec![$($val),*], $crate::qattribute::NONE)
    };
    (date: vec![$($val:expr),* $(,)?]; @$attr:ident) => {
        $crate::K::new_date_list(vec![$($val),*], k!(@attr $attr))
    };

    // Datetime list (chrono::DateTime<Utc>)
    (datetime: vec![$val:expr; $count:expr]) => {
        $crate::K::new_datetime_list(vec![$val; $count], $crate::qattribute::NONE)
    };
    (datetime: vec![$val:expr; $count:expr]; @$attr:ident) => {
        $crate::K::new_datetime_list(vec![$val; $count], k!(@attr $attr))
    };
    (datetime: vec![$($val:expr),* $(,)?]) => {
        $crate::K::new_datetime_list(vec![$($val),*], $crate::qattribute::NONE)
    };
    (datetime: vec![$($val:expr),* $(,)?]; @$attr:ident) => {
        $crate::K::new_datetime_list(vec![$($val),*], k!(@attr $attr))
    };

    // Timespan list (chrono::Duration)
    (timespan: vec![$val:expr; $count:expr]) => {
        $crate::K::new_timespan_list(vec![$val; $count], $crate::qattribute::NONE)
    };
    (timespan: vec![$val:expr; $count:expr]; @$attr:ident) => {
        $crate::K::new_timespan_list(vec![$val; $count], k!(@attr $attr))
    };
    (timespan: vec![$($val:expr),* $(,)?]) => {
        $crate::K::new_timespan_list(vec![$($val),*], $crate::qattribute::NONE)
    };
    (timespan: vec![$($val:expr),* $(,)?]; @$attr:ident) => {
        $crate::K::new_timespan_list(vec![$($val),*], k!(@attr $attr))
    };

    // Minute list (chrono::Duration)
    (minute: vec![$val:expr; $count:expr]) => {
        $crate::K::new_minute_list(vec![$val; $count], $crate::qattribute::NONE)
    };
    (minute: vec![$val:expr; $count:expr]; @$attr:ident) => {
        $crate::K::new_minute_list(vec![$val; $count], k!(@attr $attr))
    };
    (minute: vec![$($val:expr),* $(,)?]) => {
        $crate::K::new_minute_list(vec![$($val),*], $crate::qattribute::NONE)
    };
    (minute: vec![$($val:expr),* $(,)?]; @$attr:ident) => {
        $crate::K::new_minute_list(vec![$($val),*], k!(@attr $attr))
    };

    // Second list (chrono::Duration)
    (second: vec![$val:expr; $count:expr]) => {
        $crate::K::new_second_list(vec![$val; $count], $crate::qattribute::NONE)
    };
    (second: vec![$val:expr; $count:expr]; @$attr:ident) => {
        $crate::K::new_second_list(vec![$val; $count], k!(@attr $attr))
    };
    (second: vec![$($val:expr),* $(,)?]) => {
        $crate::K::new_second_list(vec![$($val),*], $crate::qattribute::NONE)
    };
    (second: vec![$($val:expr),* $(,)?]; @$attr:ident) => {
        $crate::K::new_second_list(vec![$($val),*], k!(@attr $attr))
    };

    // Time list (chrono::Duration)
    (time: vec![$val:expr; $count:expr]) => {
        $crate::K::new_time_list(vec![$val; $count], $crate::qattribute::NONE)
    };
    (time: vec![$val:expr; $count:expr]; @$attr:ident) => {
        $crate::K::new_time_list(vec![$val; $count], k!(@attr $attr))
    };
    (time: vec![$($val:expr),* $(,)?]) => {
        $crate::K::new_time_list(vec![$($val),*], $crate::qattribute::NONE)
    };
    (time: vec![$($val:expr),* $(,)?]; @$attr:ident) => {
        $crate::K::new_time_list(vec![$($val),*], k!(@attr $attr))
    };

    // ========== Atoms ==========

    (bool: $val:expr) => {
        $crate::K::new_bool($val)
    };

    (byte: $val:expr) => {
        $crate::K::new_byte($val)
    };

    (short: $val:expr) => {
        $crate::K::new_short($val)
    };

    (int: $val:expr) => {
        $crate::K::new_int($val)
    };

    (long: $val:expr) => {
        $crate::K::new_long($val)
    };

    (real: $val:expr) => {
        $crate::K::new_real($val)
    };

    (float: $val:expr) => {
        $crate::K::new_float($val)
    };

    (char: $val:expr) => {
        $crate::K::new_char($val)
    };

    (sym: $val:expr) => {{
        $crate::K::new_symbol($val.to_string())
    }};

    (string: $val:expr) => {
        $crate::K::new_string($val.to_string(), $crate::qattribute::NONE)
    };

    // String with attribute
    (string: $val:expr; @$attr:ident) => {
        $crate::K::new_string($val.to_string(), k!(@attr $attr))
    };

    // Temporal atoms
    (timestamp: $val:expr) => {
        $crate::K::new_timestamp($val)
    };

    (month: $val:expr) => {
        $crate::K::new_month($val)
    };

    (date: $val:expr) => {
        $crate::K::new_date($val)
    };

    (datetime: $val:expr) => {
        $crate::K::new_datetime($val)
    };

    (timespan: $val:expr) => {
        $crate::K::new_timespan($val)
    };

    (minute: $val:expr) => {
        $crate::K::new_minute($val)
    };

    (second: $val:expr) => {
        $crate::K::new_second($val)
    };

    (time: $val:expr) => {
        $crate::K::new_time($val)
    };

    // ========== Compound Lists ==========

    ([$($item:expr),* $(,)?]) => {
        $crate::K::new_compound_list(vec![$($item),*])
    };

    // ========== Dictionaries ==========

    (dict: $keys:expr => $values:expr) => {
        $crate::K::new_dictionary($keys, $values).expect("Failed to create dictionary")
    };

    // ========== Tables ==========

    // Table from column definitions using braces
    (table: { $($col_name:expr => $col_data:expr),* $(,)? }) => {{
        let keys = $crate::K::new_symbol_list(
            vec![$($col_name.to_string()),*],
            $crate::qattribute::NONE
        );
        let values = $crate::K::new_compound_list(vec![$($col_data),*]);
        $crate::K::new_dictionary(keys, values)
            .expect("Failed to create table dictionary")
            .flip()
            .expect("Failed to flip dictionary to table")
    }};

    // Flip a dictionary to create a table
    (flip: $dict:expr) => {
        $dict.flip().expect("Failed to flip dictionary to table")
    };

    // ========== Attribute helper ==========

    (@attr sorted) => { $crate::qattribute::SORTED };
    (@attr unique) => { $crate::qattribute::UNIQUE };
    (@attr parted) => { $crate::qattribute::PARTED };
    (@attr grouped) => { $crate::qattribute::GROUPED };
}

#[cfg(test)]
mod tests {
    use crate::*;

    #[test]
    fn test_atoms() {
        let _ = k!(bool: true);
        let _ = k!(byte: 42);
        let _ = k!(short: 42);
        let _ = k!(int: 42);
        let _ = k!(long: 42);
        let _ = k!(real: 42.5);
        let _ = k!(float: 42.5);
        let _ = k!(char: 'a');
        let _ = k!(sym: "symbol");
        let _ = k!(string: "hello");
    }

    #[test]
    fn test_lists() {
        let _ = k!(bool: vec![true, false, true]);
        let _ = k!(byte: vec![1, 2, 3]);
        let _ = k!(short: vec![1, 2, 3]);
        let _ = k!(int: vec![1, 2, 3]);
        let _ = k!(long: vec![1, 2, 3]);
        let _ = k!(real: vec![1.1, 2.2, 3.3]);
        let _ = k!(float: vec![1.1, 2.2, 3.3]);
        let _ = k!(sym: vec!["a", "b", "c"]);
        let _ = k!(string: "hello");
    }

    #[test]
    fn test_temporal_atoms() {
        use chrono::prelude::*;
        use chrono::Duration;

        let ts = NaiveDate::from_ymd_opt(2024, 1, 15)
            .unwrap()
            .and_hms_nano_opt(10, 30, 0, 123456789)
            .unwrap()
            .and_local_timezone(Utc)
            .unwrap();
        let _ = k!(timestamp: ts);
        let _ = k!(datetime: ts);

        let date = NaiveDate::from_ymd_opt(2024, 1, 15).unwrap();
        let _ = k!(date: date);
        let _ = k!(month: date);

        let _ = k!(timespan: Duration::hours(5));
        let _ = k!(minute: Duration::minutes(30));
        let _ = k!(second: Duration::seconds(90));
        let _ = k!(time: Duration::milliseconds(1000));
    }

    #[test]
    fn test_temporal_lists() {
        use chrono::prelude::*;
        use chrono::Duration;

        let dates = vec![
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            NaiveDate::from_ymd_opt(2024, 1, 2).unwrap(),
        ];
        let _ = k!(date: vec![dates[0], dates[1]]);
        let _ = k!(month: vec![dates[0], dates[1]]; @sorted);

        let durations = vec![Duration::seconds(10), Duration::seconds(20)];
        let _ = k!(timespan: vec![durations[0], durations[1]]);
        let _ = k!(minute: vec![Duration::minutes(1), Duration::minutes(2)]);
    }

    #[test]
    fn test_lists_with_attributes() {
        let _ = k!(long: vec![1, 2, 3]; @sorted);
        let _ = k!(sym: vec!["a", "b", "c"]; @unique);
    }

    #[test]
    fn test_list_repetition_syntax() {
        // Test vec![value; count] syntax for all list types
        let long_list = k!(long: vec![42; 3000]);
        assert_eq!(long_list.len(), 3000);

        let int_list = k!(int: vec![10; 100]);
        assert_eq!(int_list.len(), 100);

        let float_list = k!(float: vec![3.14; 50]);
        assert_eq!(float_list.len(), 50);

        let sym_list = k!(sym: vec!["test"; 10]);
        assert_eq!(sym_list.len(), 10);

        // Test with attributes
        let sorted_list = k!(long: vec![1; 2500]; @sorted);
        assert_eq!(sorted_list.len(), 2500);
        assert_eq!(sorted_list.get_attribute(), qattribute::SORTED);
    }

    #[test]
    fn test_compound_list() {
        let _ = k!([k!(long: 1), k!(float: 2.5), k!(sym: "symbol")]);
    }

    #[test]
    fn test_dictionary() {
        let keys = k!(int: vec![1, 2, 3]);
        let values = k!(sym: vec!["a", "b", "c"]);
        let _ = k!(dict: keys => values);
    }

    #[test]
    fn test_table() {
        let _ = k!(table: {
            "col1" => k!(int: vec![1, 2, 3]),
            "col2" => k!(float: vec![1.1, 2.2, 3.3]),
            "col3" => k!(sym: vec!["a", "b", "c"])
        });
    }
}
