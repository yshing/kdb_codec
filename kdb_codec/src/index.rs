//! Index trait implementations for K objects.
//!
//! This module provides `Index` and `IndexMut` trait implementations to enable
//! intuitive `[]` syntax for accessing K object data.
//!
//! # Examples
//!
//! ## Dictionary Access by Position
//! ```rust
//! use kdb_codec::*;
//!
//! let dict = k!(dict: k!(sym: vec!["a", "b"]) => k!(long: vec![10, 20]));
//!
//! // Access keys and values using index
//! let keys_ref = &dict[0];    // Get keys K object
//! let values_ref = &dict[1];  // Get values K object
//! ```
//!
//! ## Dictionary Lookup by Key
//! ```rust
//! use kdb_codec::*;
//!
//! let dict = k!(dict:
//!     k!(sym: vec!["apple", "banana"]) =>
//!     k!([k!(long: 10), k!(long: 20)])
//! );
//!
//! // Look up value by key
//! let key = k!(sym: "banana");
//! let value = &dict[&key];  // Returns K object with value 20
//! ```
//!
//! ## Table Column Access
//! ```rust
//! use kdb_codec::*;
//!
//! let table = k!(table: {
//!     "fruit" => k!(sym: vec!["apple"]),
//!     "price" => k!(float: vec![1.5])
//! });
//!
//! // Access columns by name
//! let fruits = &table["fruit"];
//! let prices = &table["price"];
//! ```

use crate::error::Error;
use crate::qconsts::qtype;
use crate::types::K;
use std::ops::{Index, IndexMut};

// Dictionary indexing by position (0 = keys, 1 = values)
impl Index<usize> for K {
    type Output = K;

    /// Access dictionary keys (index 0) or values (index 1).
    ///
    /// # Panics
    /// Panics if:
    /// - The K object is not a dictionary
    /// - Index is out of bounds (not 0 or 1)
    ///
    /// # Example
    /// ```
    /// use kdb_codec::*;
    ///
    /// let dict = k!(dict: k!(sym: vec!["x"]) => k!(long: vec![42]));
    ///
    /// let dict_keys = &dict[0];
    /// let dict_values = &dict[1];
    /// ```
    fn index(&self, idx: usize) -> &Self::Output {
        match self.get_type() {
            qtype::DICTIONARY | qtype::SORTED_DICTIONARY => self
                .as_vec::<K>()
                .expect("Dictionary should contain K vector")
                .get(idx)
                .expect("Dictionary index must be 0 (keys) or 1 (values)"),
            _ => panic!(
                "Index<usize> only supported for dictionaries, got type {}",
                self.get_type()
            ),
        }
    }
}

impl IndexMut<usize> for K {
    /// Mutably access dictionary keys (index 0) or values (index 1).
    ///
    /// # Panics
    /// Panics if:
    /// - The K object is not a dictionary
    /// - Index is out of bounds (not 0 or 1)
    ///
    /// # Example
    /// ```
    /// use kdb_codec::*;
    ///
    /// let mut dict = k!(dict: k!(sym: vec!["x"]) => k!(long: vec![42]));
    ///
    /// // Replace values
    /// dict[1] = k!(long: vec![100]);
    /// ```
    fn index_mut(&mut self, idx: usize) -> &mut Self::Output {
        match self.get_type() {
            qtype::DICTIONARY | qtype::SORTED_DICTIONARY => self
                .as_mut_vec::<K>()
                .expect("Dictionary should contain K vector")
                .get_mut(idx)
                .expect("Dictionary index must be 0 (keys) or 1 (values)"),
            _ => panic!(
                "IndexMut<usize> only supported for dictionaries, got type {}",
                self.get_type()
            ),
        }
    }
}

// Dictionary lookup by K object (key lookup)
impl Index<&K> for K {
    type Output = K;

    /// Look up a value in a dictionary by key.
    ///
    /// # Panics
    /// Panics if:
    /// - The K object is not a dictionary
    /// - The key is not found in the dictionary
    ///
    /// # Example
    /// ```
    /// use kdb_codec::*;
    ///
    /// let dict = k!(dict: k!(sym: vec!["a", "b", "c"]) => k!(long: vec![10, 20, 30]));
    ///
    /// let key = k!(sym: "b");
    /// let value = &dict[&key];  // Returns K object with value 20
    /// ```
    fn index(&self, key: &K) -> &Self::Output {
        self.find_value(key)
            .unwrap_or_else(|_| panic!("Key {:?} not found in dictionary", key))
    }
}

// Table column access by name (&str)
impl Index<&str> for K {
    type Output = K;

    /// Access table column by name.
    ///
    /// # Panics
    /// Panics if:
    /// - The K object is not a table
    /// - The column name does not exist
    ///
    /// # Example
    /// ```
    /// use kdb_codec::*;
    ///
    /// let table = k!(table: {
    ///     "price" => k!(float: vec![1.5])
    /// });
    ///
    /// let price_column = &table["price"];
    /// ```
    fn index(&self, column: &str) -> &Self::Output {
        self.get_column(column)
            .unwrap_or_else(|_| panic!("Column '{}' not found in table", column))
    }
}

impl IndexMut<&str> for K {
    /// Mutably access table column by name.
    ///
    /// # Panics
    /// Panics if:
    /// - The K object is not a table
    /// - The column name does not exist
    ///
    /// # Example
    /// ```
    /// use kdb_codec::*;
    ///
    /// let mut table = k!(table: {
    ///     "price" => k!(float: vec![1.5])
    /// });
    ///
    /// // Modify column
    /// table["price"] = k!(float: vec![2.0]);
    /// ```
    fn index_mut(&mut self, column: &str) -> &mut Self::Output {
        self.get_mut_column(column)
            .unwrap_or_else(|_| panic!("Column '{}' not found in table", column))
    }
}

// Safe (non-panicking) index methods
impl K {
    /// Safely access dictionary by index, returning Result instead of panicking.
    ///
    /// # Example
    /// ```
    /// use kdb_codec::*;
    ///
    /// let dict = k!(dict: k!(sym: vec!["x"]) => k!(long: vec![42]));
    ///
    /// assert!(dict.try_index(0).is_ok());
    /// assert!(dict.try_index(2).is_err());  // Out of bounds
    /// ```
    pub fn try_index(&self, idx: usize) -> Result<&K, Error> {
        match self.get_type() {
            qtype::DICTIONARY | qtype::SORTED_DICTIONARY => {
                let vec = self.as_vec::<K>()?;
                vec.get(idx)
                    .ok_or_else(|| Error::index_out_of_bounds(vec.len(), idx))
            }
            qtype::COMPOUND_LIST => {
                let vec = self.as_vec::<K>()?;
                vec.get(idx)
                    .ok_or_else(|| Error::index_out_of_bounds(vec.len(), idx))
            }
            _ => Err(Error::invalid_operation("try_index", self.get_type(), None)),
        }
    }

    /// Safely mutably access dictionary by index, returning Result instead of panicking.
    ///
    /// # Example
    /// ```
    /// use kdb_codec::*;
    ///
    /// let mut dict = k!(dict: k!(sym: vec!["x"]) => k!(long: vec![42]));
    ///
    /// if let Ok(values) = dict.try_index_mut(1) {
    ///     *values = k!(long: vec![100]);
    /// }
    /// ```
    pub fn try_index_mut(&mut self, idx: usize) -> Result<&mut K, Error> {
        match self.get_type() {
            qtype::DICTIONARY | qtype::SORTED_DICTIONARY => {
                let len = self.as_vec::<K>()?.len();
                let vec = self.as_mut_vec::<K>()?;
                vec.get_mut(idx)
                    .ok_or_else(|| Error::index_out_of_bounds(len, idx))
            }
            qtype::COMPOUND_LIST => {
                let len = self.as_vec::<K>()?.len();
                let vec = self.as_mut_vec::<K>()?;
                vec.get_mut(idx)
                    .ok_or_else(|| Error::index_out_of_bounds(len, idx))
            }
            _ => Err(Error::invalid_operation(
                "try_index_mut",
                self.get_type(),
                None,
            )),
        }
    }

    /// Safely access table column by name, returning Result instead of panicking.
    ///
    /// # Example
    /// ```
    /// use kdb_codec::*;
    ///
    /// let table = k!(table: {
    ///     "price" => k!(float: vec![1.5])
    /// });
    ///
    /// assert!(table.try_column("price").is_ok());
    /// assert!(table.try_column("nonexistent").is_err());
    /// ```
    pub fn try_column(&self, column: &str) -> Result<&K, Error> {
        self.get_column(column)
    }

    /// Safely mutably access table column by name, returning Result instead of panicking.
    ///
    /// # Example
    /// ```
    /// use kdb_codec::*;
    ///
    /// let mut table = k!(table: {
    ///     "price" => k!(float: vec![1.5])
    /// });
    ///
    /// if let Ok(price) = table.try_column_mut("price") {
    ///     *price = k!(float: vec![2.0]);
    /// }
    /// ```
    pub fn try_column_mut(&mut self, column: &str) -> Result<&mut K, Error> {
        self.get_mut_column(column)
    }

    /// Look up a value in a dictionary by key, returning Result instead of panicking.
    ///
    /// This searches for the key in the dictionary's keys and returns the corresponding value.
    ///
    /// # Example
    /// ```
    /// use kdb_codec::*;
    ///
    /// let dict = k!(dict: k!(sym: vec!["a", "b", "c"]) => k!(long: vec![10, 20, 30]));
    ///
    /// let key = k!(sym: "b");
    /// assert!(dict.try_find(&key).is_ok());
    ///
    /// let missing_key = k!(sym: "z");
    /// assert!(dict.try_find(&missing_key).is_err());
    /// ```
    pub fn try_find(&self, key: &K) -> Result<&K, Error> {
        match self.get_type() {
            qtype::DICTIONARY | qtype::SORTED_DICTIONARY => {
                let dict_vec = self.as_vec::<K>()?;
                let keys = &dict_vec[0];
                let values = &dict_vec[1];

                // Find the key in the keys list
                let key_index = Self::find_key_index(keys, key)?;

                // Get the corresponding value
                values
                    .as_vec::<K>()?
                    .get(key_index)
                    .ok_or_else(|| Error::index_out_of_bounds(values.len(), key_index))
            }
            _ => Err(Error::invalid_operation("try_find", self.get_type(), None)),
        }
    }

    /// Internal helper to find the index of a key in a dictionary's key list.
    fn find_key_index(keys: &K, target_key: &K) -> Result<usize, Error> {
        // Handle different key types
        match keys.get_type() {
            qtype::SYMBOL_LIST => {
                let target_sym = target_key.get_symbol()?;
                let key_list = keys.as_vec::<String>()?;
                key_list
                    .iter()
                    .position(|k| k == target_sym)
                    .ok_or_else(|| Error::NoSuchColumn(format!("Key '{}' not found", target_sym)))
            }
            qtype::LONG_LIST => {
                let target_long = target_key.get_long()?;
                let key_list = keys.as_vec::<i64>()?;
                key_list
                    .iter()
                    .position(|&k| k == target_long)
                    .ok_or_else(|| Error::NoSuchColumn(format!("Key {} not found", target_long)))
            }
            qtype::INT_LIST => {
                let target_int = target_key.get_int()?;
                let key_list = keys.as_vec::<i32>()?;
                key_list
                    .iter()
                    .position(|&k| k == target_int)
                    .ok_or_else(|| Error::NoSuchColumn(format!("Key {} not found", target_int)))
            }
            qtype::FLOAT_LIST => {
                let target_float = target_key.get_float()?;
                let key_list = keys.as_vec::<f64>()?;
                key_list
                    .iter()
                    .position(|&k| (k - target_float).abs() < f64::EPSILON)
                    .ok_or_else(|| Error::NoSuchColumn(format!("Key {} not found", target_float)))
            }
            _ => Err(Error::invalid_operation(
                "find_key_index",
                keys.get_type(),
                None,
            )),
        }
    }

    /// Internal helper used by Index<&K> trait.
    fn find_value(&self, key: &K) -> Result<&K, Error> {
        self.try_find(key)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::k;
    use crate::types::J;

    #[test]
    fn test_dictionary_index_read() {
        let dict = k!(dict: k!(sym: vec!["a", "b"]) => k!(long: vec![10, 20]));

        let keys_ref = &dict[0];
        let values_ref = &dict[1];

        assert_eq!(keys_ref.get_type(), qtype::SYMBOL_LIST);
        assert_eq!(values_ref.get_type(), qtype::LONG_LIST);
        assert_eq!(values_ref.as_vec::<J>().unwrap()[0], 10);
    }

    #[test]
    fn test_dictionary_index_write() {
        let mut dict = k!(dict: k!(sym: vec!["x"]) => k!(long: vec![42]));

        dict[1] = k!(long: vec![100]);

        let new_values = &dict[1];
        assert_eq!(new_values.as_vec::<J>().unwrap()[0], 100);
    }

    #[test]
    fn test_table_column_index() {
        let table = k!(table: {
            "fruit" => k!(sym: vec!["apple", "banana"]),
            "price" => k!(float: vec![1.5, 2.3])
        });

        let fruits = &table["fruit"];
        let prices = &table["price"];

        assert_eq!(fruits.get_type(), qtype::SYMBOL_LIST);
        assert_eq!(prices.get_type(), qtype::FLOAT_LIST);
    }

    #[test]
    fn test_table_column_index_mut() {
        let mut table = k!(table: {
            "price" => k!(float: vec![1.5])
        });

        table["price"] = k!(float: vec![2.0]);

        let new_prices = &table["price"];
        assert_eq!(new_prices.as_vec::<f64>().unwrap()[0], 2.0);
    }

    #[test]
    #[should_panic(expected = "Column 'nonexistent' not found")]
    fn test_table_invalid_column_panics() {
        let table = k!(table: {
            "price" => k!(float: vec![1.5])
        });

        let _ = &table["nonexistent"];
    }

    #[test]
    fn test_try_index_safe() {
        let dict = k!(dict: k!(sym: vec!["x"]) => k!(long: vec![42]));

        assert!(dict.try_index(0).is_ok());
        assert!(dict.try_index(1).is_ok());
        assert!(dict.try_index(2).is_err()); // Out of bounds
    }

    #[test]
    fn test_try_column_safe() {
        let table = k!(table: {
            "price" => k!(float: vec![1.5])
        });

        assert!(table.try_column("price").is_ok());
        assert!(table.try_column("nonexistent").is_err());
    }

    #[test]
    fn test_compound_list_try_index() {
        let list = k!([k!(long: 42), k!(float: 3.14), k!(sym: "test")]);

        assert!(list.try_index(0).is_ok());
        assert!(list.try_index(1).is_ok());
        assert!(list.try_index(2).is_ok());
        assert!(list.try_index(3).is_err());
    }

    #[test]
    fn test_dictionary_lookup_by_key() {
        // Symbol keys with compound list values
        let dict = k!(dict:
            k!(sym: vec!["apple", "banana", "cherry"]) =>
            k!([k!(long: 10), k!(long: 20), k!(long: 30)])
        );

        let key1 = k!(sym: "apple");
        let value1 = &dict[&key1];
        assert_eq!(value1.get_long().unwrap(), 10);

        let key2 = k!(sym: "cherry");
        let value2 = &dict[&key2];
        assert_eq!(value2.get_long().unwrap(), 30);
    }

    #[test]
    fn test_dictionary_lookup_int_keys() {
        let dict = k!(dict:
            k!(int: vec![1, 2, 3]) =>
            k!([k!(sym: "one"), k!(sym: "two"), k!(sym: "three")])
        );

        let key = k!(int: 2);
        let value = &dict[&key];
        assert_eq!(value.get_symbol().unwrap(), "two");
    }

    #[test]
    fn test_dictionary_lookup_long_keys() {
        let dict = k!(dict:
            k!(long: vec![100, 200, 300]) =>
            k!([k!(float: 1.1), k!(float: 2.2), k!(float: 3.3)])
        );

        let key = k!(long: 200);
        let value = &dict[&key];
        assert!((value.get_float().unwrap() - 2.2).abs() < f64::EPSILON);
    }

    #[test]
    #[should_panic(expected = "not found")]
    fn test_dictionary_lookup_missing_key() {
        let dict = k!(dict:
            k!(sym: vec!["a", "b"]) =>
            k!([k!(long: 10), k!(long: 20)])
        );

        let missing_key = k!(sym: "z");
        let _ = &dict[&missing_key]; // Should panic
    }

    #[test]
    fn test_try_find_safe() {
        let dict = k!(dict:
            k!(sym: vec!["x", "y", "z"]) =>
            k!([k!(long: 1), k!(long: 2), k!(long: 3)])
        );

        let key1 = k!(sym: "y");
        assert!(dict.try_find(&key1).is_ok());
        assert_eq!(dict.try_find(&key1).unwrap().get_long().unwrap(), 2);

        let missing_key = k!(sym: "missing");
        assert!(dict.try_find(&missing_key).is_err());
    }
}
