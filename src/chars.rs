use std::num::ParseIntError;

/// Error type of [unescape](unescape).
#[derive(Debug, PartialEq)]
pub enum UnescapeError {
    InvalidEscape {
        escape: String,
        index: usize,
        string: String,
    },
    InvalidUnicode {
        source: ParseUnicodeError,
        index: usize,
        string: String,
    },
}

/// Source error type of [UnescapeError::InvalidUnicode](UnescapeError::InvalidUnicode).
#[derive(Debug, PartialEq)]
pub enum ParseUnicodeError {
    BraceNotFound,
    ParseHexFailed {
        source: ParseIntError,
        string: String,
    },
    ParseUnicodeFailed { value: u32 },
}

pub fn unescape(s: &str) -> Result<String, UnescapeError> {
    let mut in_single_quote = false;
    let mut in_double_quote = false;

    let mut chars = s.chars().enumerate();

    let mut res = String::with_capacity(s.len());

    while let Some((idx, c)) = chars.next() {
        // when in a single quote, no escapes are possible
        if in_single_quote {
            if c == '\'' {
                in_single_quote = false;
                continue;
            }
        } else if in_double_quote {
            if c == '"' {
                in_double_quote = false;
                continue;
            }

            if c == '\\' {
                match chars.next() {
                    None => {
                        return Err(UnescapeError::InvalidEscape {
                            escape: format!("{}", c),
                            index: idx,
                            string: String::from(s),
                        });
                    }
                    Some((idx, c2)) => {
                        res.push(match c2 {
                            'a' => '\u{07}',
                            'b' => '\u{08}',
                            'v' => '\u{0B}',
                            'f' => '\u{0C}',
                            'n' => '\n',
                            'r' => '\r',
                            't' => '\t',
                            'e' | 'E' => '\u{1B}',
                            '\\' => '\\',
                            '\'' => '\'',
                            '"' => '"',
                            '$' => '$',
                            '`' => '`',
                            ' ' => ' ',
                            'u' => parse_unicode(&mut chars).map_err(|x| {
                                UnescapeError::InvalidUnicode {
                                    source: x,
                                    index: idx,
                                    string: String::from(s),
                                }
                            })?,
                            _ => {
                                return Err(UnescapeError::InvalidEscape {
                                    escape: format!("{}{}", c, c2),
                                    index: idx,
                                    string: String::from(s),
                                });
                            }
                        });
                        continue;
                    }
                };
            }
        } else if c == '\'' {
            in_single_quote = true;
            continue;
        } else if c == '"' {
            in_double_quote = true;
            continue;
        }

        res.push(c);
    }

    Ok(res)
}

// parse_unicode takes an iterator over characters and attempts to extract a single unicode
// character from it.
// It parses escapes of the form '\u{65b9}', but this internal helper function expects the cursor
// to be advanced to between the 'u' and '{'.
// It also expects to be passed an iterator which includes the index for the purpose of advancing
// it  as well, such as is produced by enumerate.
fn parse_unicode<I>(chars: &mut I) -> Result<char, ParseUnicodeError>
    where
        I: Iterator<Item = (usize, char)>,
{
    match chars.next() {
        Some((_, '{')) => {}
        _ => {
            return Err(ParseUnicodeError::BraceNotFound);
        }
    }

    let unicode_seq: String = chars
        .take_while(|&(_, c)| c != '}')
        .map(|(_, c)| c)
        .collect();

    u32::from_str_radix(&unicode_seq, 16)
        .map_err(|e| ParseUnicodeError::ParseHexFailed {
            source: e,
            string: unicode_seq,
        })
        .and_then(|u| {
            char::from_u32(u).ok_or_else(|| ParseUnicodeError::ParseUnicodeFailed { value: u })
        })
}