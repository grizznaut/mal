use crate::types::MalType;
use crate::{hashmap, list, vector};
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
    pub fn next(&mut self) -> Result<Token, &'static str> {
        let token = self.peek()?;
        self.position += 1;
        Ok(token)
    }

    /// just returns the token at the current position
    pub fn peek(&self) -> Result<Token, &'static str> {
        Ok(self
            .tokens
            .get(self.position)
            .ok_or("Reader position out of bounds")?
            .to_string())
    }
}

/// This function will call tokenize and then create a new Reader object instance with the tokens.
/// Then it will call read_form with the Reader instance.
pub fn read_str(s: String) -> Result<MalType, &'static str> {
    let mut reader = Reader::new(tokenize(s));
    read_form(&mut reader)
}

/// This function will take a single string and return an array/list of all the tokens (strings) in it.
/// The following regular expression (PCRE) will match all mal tokens.
/// [\s,]*(~@|[\[\]{}()'`~^@]|"(?:\\.|[^\\"])*"?|;.*|[^\s\[\]{}('"`,;)]*)
fn tokenize(s: String) -> Vec<Token> {
    let re = Regex::new(r#"[\s,]*(~@|[\[\]{}()'`~^@]|"(?:\\.|[^\\"])*"?|;.*|[^\s\[\]{}('"`,;)]*)"#)
        .unwrap();
    let tokens: Vec<String> = re
        .captures_iter(&s.trim())
        .map(|caps| String::from(&caps[1]))
        .collect();
    // dbg!(&tokens);
    tokens
}

/// This function will peek at the first token in the Reader object and switch on the first character of that token.
/// If the character is a left paren then read_list is called with the Reader object.
/// Otherwise, read_atom is called with the Reader Object.
/// The return value from read_form is a mal data type.
fn read_form(reader: &mut Reader) -> Result<MalType, &'static str> {
    match reader.peek()?.as_str() {
        "(" => read_list(reader, ")"),
        ")" => return Err("Unexpected ')'"),
        "[" => read_list(reader, "]"),
        "]" => return Err("Unexpected ']'"),
        "{" => read_list(reader, "}"),
        "}" => return Err("Unexpected '}'"),
        _ => read_atom(reader),
    }
}

/// This function will repeatedly call read_form with the Reader object until it encounters a ')' token
/// (if it reach EOF before reading a ')' then that is an error).
/// It accumulates the results into a List type.
fn read_list(reader: &mut Reader, end: &str) -> Result<MalType, &'static str> {
    let mut list: Vec<MalType> = vec![];

    // skip opening brace
    reader.next()?;

    loop {
        let token = match reader.peek() {
            Ok(t) => t,
            _ => return Err("Unexpected EOF"),
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
        _ => Err("Unknown end value"),
    }
}

/// This function will look at the contents of the token and return the appropriate scalar (simple/single) data type value.
/// Initially, you can just implement numbers (integers) and symbols.
fn read_atom(reader: &mut Reader) -> Result<MalType, &'static str> {
    let token = reader.next()?;
    MalType::try_from(token)
}

impl TryFrom<Token> for MalType {
    type Error = &'static str;

    fn try_from(token: Token) -> Result<Self, Self::Error> {
        match token.as_str() {
            "nil" => Ok(MalType::Nil),
            "true" => Ok(MalType::Bool(true)),
            "false" => Ok(MalType::Bool(false)),
            _ => match token.parse::<i64>() {
                Ok(int) => Ok(MalType::Int(int)),
                Err(_) => Ok(MalType::Symbol(token)),
            },
        }
    }
}
