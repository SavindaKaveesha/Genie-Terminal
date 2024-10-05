/*!
# Word Dictionary

This crate provides a data structure for word mapping. It can be used for language translation.

## Examples

```rust
use word_dictionary::Dictionary;

let mut dictionary = Dictionary::new("tests/data/dictionary.txt"); // input a dictionary file

// dictionary.read_data().unwrap(); // if the dictionary file already exists

dictionary.add_edit("Althasol", "阿爾瑟索").unwrap();
dictionary.add_edit("Aldun", "奧爾敦").unwrap();
dictionary.add_edit("Alduin", "阿爾杜因").unwrap();
dictionary.add_edit("Alduin", "奥杜因").unwrap();

assert_eq!("阿爾瑟索", dictionary.get_right(dictionary.find_left_strictly("Althasol", 0).unwrap()).unwrap());
assert_eq!("奧爾敦", dictionary.get_right(dictionary.find_left("dun", 0).unwrap()).unwrap());
assert_eq!("奥杜因", dictionary.get_right(dictionary.find_left("Alduin", 0).unwrap()).unwrap());
assert_eq!("阿爾杜因 --> 奥杜因", dictionary.get_all_right_to_string(dictionary.find_left("Alduin", 0).unwrap()).unwrap());

// The dictionary file now would be
/*
Alduin = 阿爾杜因 --> 奥杜因
Aldun = 奧爾敦
Althasol = 阿爾瑟索
*/
```
 */

use std::fs::File;
use std::io::{BufRead, BufReader, ErrorKind, Write};
use std::path::PathBuf;

use std::error::Error;
use std::fmt::{self, Display, Formatter};
use std::io;

use std::collections::HashMap;

#[derive(Debug)]
pub enum BrokenReason {
    BadLeftString,
    NoRightString,
    BadRightString {
        right_string: String,
    },
    Duplicated {
        another_left_string: String,
    },
}

#[derive(Debug)]
pub enum ReadError {
    IOError(io::Error),
    Broken {
        line: usize,
        left_string: String,
        reason: BrokenReason,
    },
}

impl From<io::Error> for ReadError {
    #[inline]
    fn from(error: io::Error) -> Self {
        ReadError::IOError(error)
    }
}

impl Display for ReadError {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), fmt::Error> {
        match self {
            ReadError::IOError(err) => Display::fmt(&err, f),
            ReadError::Broken {
                line,
                left_string,
                reason,
            } => {
                f.write_fmt(format_args!("broken at line {}, ", line))?;

                match reason {
                    BrokenReason::BadLeftString => {
                        f.write_fmt(format_args!(
                            "the left string {:?} is not correct",
                            left_string
                        ))
                    }
                    BrokenReason::NoRightString => {
                        f.write_fmt(format_args!(
                            "expected a \"=\" after the left string {:?} to concatenate a right string",
                            left_string
                        ))
                    }
                    BrokenReason::BadRightString {
                        right_string,
                    } => {
                        f.write_fmt(format_args!(
                            "the right string {:?} is not correct",
                            right_string
                        ))
                    }
                    BrokenReason::Duplicated {
                        another_left_string,
                    } => {
                        if left_string == another_left_string {
                            f.write_fmt(format_args!(
                                "the left string {:#?} is duplicated",
                                left_string
                            ))
                        } else {
                            f.write_fmt(format_args!(
                                "the left string {:#?} and {:#?} are duplicated",
                                left_string, another_left_string
                            ))
                        }
                    }
                }
            }
        }
    }
}

impl Error for ReadError {}

#[derive(Debug)]
pub enum WriteError {
    IOError(io::Error),
    BadLeftString,
    BadRightString,
    Duplicated,
    Same,
}

impl From<io::Error> for WriteError {
    #[inline]
    fn from(error: io::Error) -> Self {
        WriteError::IOError(error)
    }
}

impl Display for WriteError {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), fmt::Error> {
        match self {
            WriteError::IOError(err) => Display::fmt(&err, f),
            WriteError::BadLeftString => f.write_str("the left word is not correct"),
            WriteError::BadRightString => f.write_str("the right word is not correct"),
            WriteError::Duplicated => {
                f.write_str("the pair of the left word and the right word is duplicated")
            }
            WriteError::Same => f.write_str("the left word is equal to the right word"),
        }
    }
}

impl Error for WriteError {}

use trim_in_place::TrimInPlace;


#[derive(Debug)]
pub struct Dictionary {
    /// The path of the dictionary file.
    path: PathBuf,
    /// Left data.
    left: Vec<String>,
    /// Right data.
    right: Vec<Vec<String>>,
}

#[derive(Debug)]
pub struct SingleItem {
    /// Left data.
    left: Option<String>,
    /// Right data.
    right: Vec<String>
}

impl Dictionary {
    /// Create a new `Dictionary` instance. But not read the file data. Use the `read_data` method to read data file the input file.
    #[inline]
    pub fn new<P: Into<PathBuf>>(path: P) -> Dictionary {
        Dictionary {
            path: path.into(),
            left: Vec::new(),
            right: Vec::new(),
        }
    }
}

impl Dictionary {
    /// Get the count of words.
    #[inline]
    pub fn count(&self) -> usize {
        debug_assert_eq!(self.left.len(), self.right.len());

        self.left.len()
    }

    /// Get the all right words.
    #[inline]
    pub fn get_all_right(&self, index: usize) -> Option<&[String]> {
        self.right.get(index).map(|v| v.as_slice())
    }

    /// Get the all keys of words.
    #[inline]
    pub fn get_all_right_keys(&self, index: usize) -> Option<&String> {
        self.left.get(index)
    }

    /// Get the all right words with their keys.
    #[inline]
    pub fn get_all_right_with_keys(&self, index: usize) -> SingleItem {
        SingleItem {
            left: self.left.get(index).cloned(),
            right: self.right.get(index).cloned().unwrap()
        }
    }

    /// Get the all right words.
    #[inline]
    pub fn get_all_right_to_string(&self, index: usize) -> Option<String> {
        self.right.get(index).map(|v| v.join(" --> "))
    }

    /// Get the last right word at a specific index.
    #[inline]
    pub fn get_right(&self, index: usize) -> Option<&str> {
        match self.right.get(index) {
            Some(v) => v.last().map(|s| s.as_str()),
            None => None,
        }
    }

    /// Get the left word at a specific index
    #[inline]
    pub fn get_left(&self, index: usize) -> Option<&str> {
        self.left.get(index).map(|s| s.as_str())
    }
}

impl Dictionary {
    /// Find a word by a keyword.
    #[inline]
    pub fn find_left_strictly<S: AsRef<str>>(&self, s: S, mut start_index: usize) -> Option<usize> {
        let size = self.count();

        if size == 0 {
            return None;
        }

        start_index %= size;

        let s = s.as_ref();

        for _ in 0..size {
            let tmp = &self.left[start_index];

            if tmp.eq_ignore_ascii_case(s) {
                return Some(start_index);
            }

            start_index += 1;

            if start_index == size {
                start_index = 0;
            }
        }

        None
    }

    #[inline]
    pub fn find_pairs<S: AsRef<str>>(&self, keyword: S) -> HashMap<String, Vec<String>> {
        let mut output = HashMap::new();

        let key_vector = self.find_left(keyword, 0).unwrap();

        for key in key_vector {
            let single_item  = self.get_all_right_with_keys(key);
            output.insert(single_item.left.unwrap(), single_item.right);
        }

        return output;
    }


    /// Find a word by a keyword.
    #[inline]
    pub fn find_left<S: AsRef<str>>(&self, s: S, mut start_index: usize) -> Option<Vec<usize>> {
        let size = self.count();

        if size == 0 {
            return None;
        }

        start_index %= size;

        let s = s.as_ref();

        let s_upper_case = s.to_uppercase();
        let s_lower_case = s.to_lowercase();

        let mut vec: Vec<usize> = Vec::new();

        for _ in 0..size {

            if vec.len() == 50 { // Only display max 50 suggestions
                break;
            }

            let tmp = &self.left[start_index];

            let tmp_upper_case = tmp.to_uppercase();

            if tmp_upper_case.contains(&s_upper_case) && !vec.contains(&start_index) {
                vec.push(start_index);
            }

            let tmp_lower_case = tmp.to_lowercase();

            if tmp_lower_case.contains(&s_lower_case) && !vec.contains(&start_index) {
                vec.push(start_index);
            }

            start_index += 1;

            if start_index == size {
                start_index = 0;
            }
        }

        return Some(vec);
    }

    /// Find a word by a keyword.
    #[inline]
    pub fn find_right_strictly<S: AsRef<str>>(
        &self,
        s: S,
        mut start_index: usize,
    ) -> Option<usize> {
        let size = self.count();

        if size == 0 {
            return None;
        }

        start_index %= size;

        let s = s.as_ref();

        for _ in 0..size {
            for tmp in self.right[start_index].iter().rev() {
                if tmp.eq_ignore_ascii_case(s) {
                    return Some(start_index);
                }
            }

            start_index += 1;

            if start_index == size {
                start_index = 0;
            }
        }

        None
    }

    /// Find a word by a keyword.
    #[inline]
    pub fn find_right<S: AsRef<str>>(&self, s: S, mut start_index: usize) -> Option<usize> {
        let size = self.count();

        if size == 0 {
            return None;
        }

        start_index %= size;

        let s = s.as_ref();

        let s_upper_case = s.to_uppercase();
        let s_lower_case = s.to_lowercase();

        for _ in 0..size {
            for tmp in self.right[start_index].iter().rev() {
                let tmp_upper_case = tmp.to_uppercase();

                if tmp_upper_case.contains(&s_upper_case) {
                    return Some(start_index);
                }

                let tmp_lower_case = tmp.to_lowercase();

                if tmp_lower_case.contains(&s_lower_case) {
                    return Some(start_index);
                }
            }

            start_index += 1;

            if start_index == size {
                start_index = 0;
            }
        }

        None
    }
}

impl Dictionary {
    /// Read the dictionary from the dictionary file.
    pub fn read_data(&mut self) -> Result<(), ReadError> {
        let file = match File::open(&self.path) {
            Ok(file) => file,
            Err(err) if err.kind() == ErrorKind::NotFound => {
                // it is okay with a file not found error
                return Ok(());
            }
            Err(err) => return Err(err.into()),
        };

        let mut reader = BufReader::new(file);

        let mut buffer = String::new();

        let mut line_counter = 1;

        loop {
            buffer.clear();

            let c = reader.read_line(&mut buffer)?;

            if c == 0 {
                break;
            }

            buffer.trim_in_place();

            if buffer.is_empty() {
                continue;
            }

            let mut tokenizer = buffer.split('=');

            let left_string = tokenizer.next().unwrap();

            if left_string.contains("-->") {
                return Err(ReadError::Broken {
                    line: line_counter,
                    left_string: String::from(left_string),
                    reason: BrokenReason::BadLeftString,
                });
            }

            let left_string = left_string.trim_end();

            // the format of the left string has been checked

            if let Some(index) = self.find_left_strictly(left_string, 0) {
                return Err(ReadError::Broken {
                    line: line_counter,
                    left_string: String::from(left_string),
                    reason: BrokenReason::Duplicated {
                        another_left_string: String::from(self.left[index].as_str()),
                    },
                });
            }

            let right_string = match tokenizer.next() {
                Some(right_string) => right_string,
                None => {
                    return Err(ReadError::Broken {
                        line: line_counter,
                        left_string: String::from(left_string),
                        reason: BrokenReason::NoRightString,
                    })
                }
            };

            if tokenizer.next().is_some() {
                return Err(ReadError::Broken {
                    line: line_counter,
                    left_string: String::from(left_string),
                    reason: BrokenReason::BadRightString {
                        right_string: String::from(right_string),
                    },
                });
            }

            let mut right_strings: Vec<String> = Vec::with_capacity(1);

            for s in right_string.split("-->").map(|s| s.trim()) {
                if s.is_empty() {
                    return Err(ReadError::Broken {
                        line: line_counter,
                        left_string: String::from(left_string),
                        reason: BrokenReason::BadRightString {
                            right_string: String::from(right_string),
                        },
                    });
                }

                right_strings.push(String::from(s));
            }

            self.left.push(String::from(left_string));
            self.right.push(right_strings);

            line_counter += 1;
        }

        Ok(())
    }
}

impl Dictionary {
    /// Write this dictionary to its dictionary file.
    pub fn write_data(&mut self) -> Result<(), WriteError> {
        let mut file = File::create(&self.path)?;

        let size = self.count();

        if size > 0 {
            let size_dec = size - 1;

            // When doing exchange sort, it also writes data to file.
            for i in 0..size_dec {
                let mut left = self.left[i].to_uppercase();

                for j in (i + 1)..size {
                    let left_2 = self.left[j].to_uppercase();

                    if left > left_2 {
                        self.left.swap(i, j);

                        self.right.swap(i, j);

                        left = left_2;
                    }
                }

                writeln!(file, "{} = {}", self.left[i], self.right[i].join(" --> "))?;
            }

            write!(file, "{} = {}", self.left[size_dec], self.right[size_dec].join(" --> "))?;
        }

        Ok(())
    }

    /// Delete a word.
    #[inline]
    pub fn delete(&mut self, index: usize) -> Result<bool, WriteError> {
        if index < self.count() {
            self.left.remove(index);
            self.right.remove(index);

            self.write_data()?;

            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Add or edit a word. If the left word exists, then update it, and return `Ok(false)`.
    pub fn add_edit<L: AsRef<str>, R: AsRef<str>>(
        &mut self,
        left: L,
        right: R,
    ) -> Result<bool, WriteError> {
        let left = left.as_ref().trim();
        let right = right.as_ref().trim();

        if left.contains("-->") || left.contains('=') {
            Err(WriteError::BadLeftString)
        } else if right.contains("-->") || right.contains('=') {
            Err(WriteError::BadRightString)
        } else if left == right {
            Err(WriteError::Same)
        } else if let Some(index) = self.find_left_strictly(left, 0) {
            if self.get_right(index).unwrap() == right {
                Err(WriteError::Duplicated)
            } else {
                self.right.get_mut(index).unwrap().push(String::from(right));

                self.write_data()?;

                Ok(false)
            }
        } else {
            self.left.push(String::from(left));
            self.right.push(vec![String::from(right)]);

            self.write_data()?;

            Ok(true)
        }
    }
}
