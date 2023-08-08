use crate::core::KEYWORD_PREFIX;
use crate::errors::MalErr;
use crate::types::MalType;
use crate::{hashmap, list, vector};
use lazy_static::lazy_static;
use regex::Regex;

type Token = String;

pub struct Reader {
    tokens: Vec<Token>,
    position: usize,
}

impl Reader {
    /// create a new reader instance with tokens
    pub fn new(tokens: Vec<Token>) -> Self {
        Self {
            tokens,
            position: 0,
        }
    }

    /// returns the token at the current position and increments the position
    pub fn next(&mut self) -> Result<Token, MalErr> {
        let token = self.peek()?;
        self.position += 1;
        Ok(token)
    }

    /// just returns the token at the current position
    pub fn peek(&self) -> Result<Token, MalErr> {
        Ok(self
            .tokens
            .get(self.position)
            .ok_or(MalErr::ReadErr("Reader position out of bounds".to_string()))?
            .to_string())
    }
}

/// This function will call tokenize and then create a new Reader object instance with the tokens.
/// Then it will call read_form with the Reader instance.
pub fn read_str(s: String) -> Result<MalType, MalErr> {
    let mut reader = Reader::new(tokenize(s));
    read_form(&mut reader)
}

lazy_static! {
    static ref RE: Regex =
        Regex::new(r#"[\s,]*(~@|[\[\]{}()'`~^@]|"(?:\\.|[^\\"])*"?|;.*|[^\s\[\]{}('"`,;)]*)"#)
            .unwrap();
}
/// This function will take a single string and return an array/list of all the tokens (strings) in it.
fn tokenize(s: String) -> Vec<Token> {
    let tokens: Vec<String> = RE
        .captures_iter(&s.trim())
        .filter_map(|caps| {
            if caps[1].starts_with(";") {
                None
            } else {
                Some(String::from(&caps[1]))
            }
        })
        .collect();
    tokens
}

/// This function will peek at the first token in the Reader object and switch on the first character of that token.
/// If the character is a left paren then read_list is called with the Reader object.
/// Otherwise, read_atom is called with the Reader Object.
/// The return value from read_form is a mal data type.
fn read_form(reader: &mut Reader) -> Result<MalType, MalErr> {
    match reader.peek()?.as_str() {
        "(" => read_list(reader, ")"),
        ")" => return Err(MalErr::ReadErr("Unexpected ')'".to_string())),
        "[" => read_list(reader, "]"),
        "]" => return Err(MalErr::ReadErr("Unexpected ']'".to_string())),
        "{" => read_list(reader, "}"),
        "}" => return Err(MalErr::ReadErr("Unexpected '}'".to_string())),
        "@" => {
            reader.next()?;
            Ok(list!(
                MalType::Symbol("deref".to_string()),
                read_form(reader)?
            ))
        }
        _ => read_atom(reader),
    }
}

/// This function will repeatedly call read_form with the Reader object until it encounters a ')' token
/// (if it reach EOF before reading a ')' then that is an error).
/// It accumulates the results into a List type.
fn read_list(reader: &mut Reader, end: &str) -> Result<MalType, MalErr> {
    let mut list: Vec<MalType> = vec![];

    // skip opening brace
    reader.next()?;

    loop {
        let token = match reader.peek() {
            Ok(t) => t,
            _ => return Err(MalErr::ReadErr("Unexpected EOF".to_string())),
        };
        if token == end {
            break;
        }
        let next = read_form(reader)?;
        list.push(next);
    }

    // skip closing brace
    reader.next()?;

    match end {
        ")" => Ok(list!(list)),
        "]" => Ok(vector!(list)),
        "}" => hashmap!(list),
        _ => Err(MalErr::ReadErr("Unknown end value".to_string())),
    }
}

/// This function will look at the contents of the token and return the appropriate scalar (simple/single) data type value.
/// Initially, you can just implement numbers (integers) and symbols.
fn read_atom(reader: &mut Reader) -> Result<MalType, MalErr> {
    let token = reader.next()?;
    MalType::try_from(token)
}

lazy_static! {
    static ref INT_RE: Regex = Regex::new(r"^-?[0-9]+$").unwrap();
    static ref STR_RE: Regex = Regex::new(r#""(?:\\.|[^\\"])*""#).unwrap();
}
impl TryFrom<Token> for MalType {
    type Error = MalErr;

    fn try_from(token: Token) -> Result<Self, Self::Error> {
        match token.as_str() {
            "nil" => Ok(MalType::Nil),
            "true" => Ok(MalType::Bool(true)),
            "false" => Ok(MalType::Bool(false)),
            _ => {
                if INT_RE.is_match(&token) {
                    Ok(MalType::Int(token.parse().unwrap()))
                } else if STR_RE.is_match(&token) {
                    Ok(MalType::Str(read_str_transform(&token)))
                } else if token.starts_with('"') {
                    Err(MalErr::ReadErr("Unbalanced string".to_string()))
                } else if token.starts_with(':') {
                    Ok(MalType::Str(format!("{}{}", KEYWORD_PREFIX, &token[1..])))
                } else {
                    Ok(MalType::Symbol(token))
                }
            }
        }
    }
}

lazy_static! {
    static ref UNESCAPE_RE: Regex = Regex::new(r#"\\(.)"#).unwrap();
}
fn read_str_transform(s: &str) -> String {
    // remove quotes
    let t = &s[1..s.len() - 1];
    // a backslash followed by a doublequote is translated into a plain doublequote character,
    // a backslash followed by "n" is translated into a newline,
    // and a backslash followed by another backslash is translated into a single backslash
    UNESCAPE_RE
        .replace_all(&t, |caps: &regex::Captures| {
            format!("{}", if &caps[1] == "n" { "\n" } else { &caps[1] })
        })
        .to_string()
}
