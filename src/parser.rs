use crate::value::Value;
use crate::error::{Result, SchemeError};
use std::cell::RefCell;
use std::collections::HashMap;
use std::iter::Peekable;
use std::rc::Rc;
use std::str::Chars;

enum Token {
    LParen,     // (
    RParen,     // )
    LBracket,   // [
    RBracket,   // ]
    LBrace,     // {
    RBrace,     // }
    Quote,      // '
    Symbol(String),
    Integer(i64),
    Bool(bool),
    String(String),
    Colon,      // :
    Comma,      // ,
    Dot,        // . (Currently unused, could be for improper lists later)
}

// Very basic tokenizer
fn tokenize(input: &str) -> Result<Vec<Token>> {
    let mut tokens = Vec::new();
    let mut chars = input.chars().peekable();

    while let Some(&c) = chars.peek() {
        match c {
            '(' => { tokens.push(Token::LParen); chars.next(); }
            ')' => { tokens.push(Token::RParen); chars.next(); }
            '[' => { tokens.push(Token::LBracket); chars.next(); }
            ']' => { tokens.push(Token::RBracket); chars.next(); }
            '{' => { tokens.push(Token::LBrace); chars.next(); }
            '}' => { tokens.push(Token::RBrace); chars.next(); }
            '\'' => { tokens.push(Token::Quote); chars.next(); }
            ':' => { tokens.push(Token::Colon); chars.next(); }
            ',' => { tokens.push(Token::Comma); chars.next(); }
            '.' => { tokens.push(Token::Dot); chars.next(); } // Keep for potential future use
            '"' => { // String literal
                chars.next(); // Consume "
                let mut s = String::new();
                while let Some(&next_c) = chars.peek() {
                     if next_c == '"' {
                        chars.next(); // Consume "
                        break;
                    } else if next_c == '\\' { // Handle basic escape
                        chars.next(); // consume \
                        if let Some(escaped_c) = chars.next() {
                             match escaped_c {
                                'n' => s.push('\n'),
                                't' => s.push('\t'),
                                '\\' => s.push('\\'),
                                '"' => s.push('"'),
                                _ => return Err(SchemeError::Parser(format!("Invalid escape sequence: \\{}", escaped_c))),
                            }
                        } else {
                             return Err(SchemeError::Parser("Unterminated string literal after escape".to_string()));
                        }
                    }
                    else {
                        s.push(next_c);
                        chars.next();
                    }
                }
                 // Check if string was terminated
                if chars.peek().is_none() && !input.ends_with('"') {
                    // This check is tricky with escapes, refine if needed
                     // return Err(SchemeError::Parser("Unterminated string literal".to_string()));
                }
                tokens.push(Token::String(s));
            }
            c if c.is_whitespace() => { chars.next(); } // Skip whitespace
            c if c.is_digit(10) || (c == '-' && chars.clone().nth(1).map_or(false, |nc| nc.is_digit(10))) => { // Integer
                let mut num_str = String::new();
                if c == '-' {
                    num_str.push(chars.next().unwrap());
                }
                while let Some(&next_c) = chars.peek() {
                    if next_c.is_digit(10) {
                        num_str.push(chars.next().unwrap());
                    } else {
                        break;
                    }
                }
                match num_str.parse::<i64>() {
                    Ok(n) => tokens.push(Token::Integer(n)),
                    Err(_) => return Err(SchemeError::Parser(format!("Invalid integer literal: {}", num_str))),
                }
            }
            ';' => { // Comment: skip till end of line
                 while let Some(next_c) = chars.next() {
                    if next_c == '\n' { break; }
                }
            }
            '#' => { // Booleans (#t, #f)
                chars.next(); // Consume #
                match chars.next() {
                    Some('t') => tokens.push(Token::Bool(true)),
                    Some('f') => tokens.push(Token::Bool(false)),
                    Some(other) => return Err(SchemeError::Parser(format!("Invalid boolean literal: #{}", other))),
                    None => return Err(SchemeError::Parser("Incomplete boolean literal: #".to_string())),
                }
            }
            _ => { // Symbol
                let mut sym = String::new();
                while let Some(&next_c) = chars.peek() {
                    if next_c.is_whitespace() || "()[]{}:,'".contains(next_c) {
                        break;
                    }
                    sym.push(chars.next().unwrap());
                }
                if !sym.is_empty() {
                    tokens.push(Token::Symbol(sym));
                } else {
                    // This case should ideally not be reached if input is valid
                    chars.next(); // Consume the unexpected character to avoid infinite loop
                    return Err(SchemeError::Parser(format!("Unexpected character: {}", c)));
                }
            }
        }
    }
    Ok(tokens)
}


fn parse_expr<'a, I>(tokens: &mut Peekable<I>) -> Result<Value>
where
    I: Iterator<Item = &'a Token>,
{
    let token = tokens.next().ok_or_else(|| SchemeError::Parser("Unexpected end of input".to_string()))?;

    match token {
        Token::LParen => parse_list(tokens),
        Token::LBracket => parse_array(tokens),
        Token::LBrace => parse_map(tokens),
        Token::Quote => {
            let expr = parse_expr(tokens)?;
            Ok(Value::List(vec![Value::Symbol("quote".to_string()), expr]))
        }
        Token::RParen => Err(SchemeError::Parser("Unexpected ')'".to_string())),
        Token::RBracket => Err(SchemeError::Parser("Unexpected ']'".to_string())),
        Token::RBrace => Err(SchemeError::Parser("Unexpected '}'".to_string())),
        Token::Colon => Err(SchemeError::Parser("Unexpected ':'".to_string())),
        Token::Comma => Err(SchemeError::Parser("Unexpected ','".to_string())),
        Token::Dot => Err(SchemeError::Parser("Unexpected '.'".to_string())), // Handle later if needed
        Token::Symbol(s) => Ok(Value::Symbol(s.clone())),
        Token::Integer(n) => Ok(Value::Integer(*n)),
        Token::Bool(b) => Ok(Value::Bool(*b)),
        Token::String(s) => Ok(Value::String(s.clone())),
    }
}

fn parse_list<'a, I>(tokens: &mut Peekable<I>) -> Result<Value>
where
    I: Iterator<Item = &'a Token>,
{
    let mut list = Vec::new();
    while let Some(token) = tokens.peek() {
        match token {
            Token::RParen => {
                tokens.next(); // Consume ')'
                return Ok(Value::List(list));
            }
            _ => {
                let expr = parse_expr(tokens)?;
                list.push(expr);
            }
        }
    }
    Err(SchemeError::Parser("Unmatched '('".to_string()))
}


fn parse_array<'a, I>(tokens: &mut Peekable<I>) -> Result<Value>
where
    I: Iterator<Item = &'a Token>,
{
    let mut arr = Vec::new();
    let mut expect_comma = false;

    // Handle empty array []
    if let Some(Token::RBracket) = tokens.peek() {
        tokens.next(); // Consume ']'
        return Ok(Value::Array(Rc::new(RefCell::new(arr))));
    }


    while let Some(token) = tokens.peek() {
         match token {
            Token::RBracket => {
                tokens.next(); // Consume ']'
                return Ok(Value::Array(Rc::new(RefCell::new(arr))));
            }
             Token::Comma => {
                if !expect_comma {
                     return Err(SchemeError::Parser("Unexpected comma in array literal".to_string()));
                }
                 tokens.next(); // Consume ','
                expect_comma = false;
                // Allow trailing comma
                if let Some(Token::RBracket) = tokens.peek() {
                    continue;
                }
            }
            _ => {
                 if expect_comma {
                     return Err(SchemeError::Parser("Expected comma or ']' in array literal".to_string()));
                 }
                let expr = parse_expr(tokens)?;
                arr.push(expr);
                expect_comma = true;
            }
        }
    }
     Err(SchemeError::Parser("Unmatched '['".to_string()))
}


fn parse_map<'a, I>(tokens: &mut Peekable<I>) -> Result<Value>
where
    I: Iterator<Item = &'a Token>,
{
    let mut map = HashMap::new();
    let mut expect_comma = false; // Expect comma between pairs
    let mut expect_value = false; // Expect value after colon
    let mut current_key: Option<String> = None;

     // Handle empty map {}
    if let Some(Token::RBrace) = tokens.peek() {
        tokens.next(); // Consume '}'
        return Ok(Value::Map(Rc::new(RefCell::new(map))));
    }

    while let Some(token) = tokens.peek() {
        match token {
            Token::RBrace => {
                if expect_value {
                     return Err(SchemeError::Parser("Expected value before '}' in map literal".to_string()));
                }
                 if current_key.is_some() {
                     return Err(SchemeError::Parser("Expected ':' and value before '}' in map literal".to_string()));
                 }
                tokens.next(); // Consume '}'
                return Ok(Value::Map(Rc::new(RefCell::new(map))));
            }
            Token::Comma => {
                 if !expect_comma {
                     return Err(SchemeError::Parser("Unexpected comma in map literal".to_string()));
                 }
                 if expect_value || current_key.is_some() {
                      return Err(SchemeError::Parser("Unexpected comma after key or colon in map literal".to_string()));
                 }
                tokens.next(); // Consume ','
                expect_comma = false;
                 // Allow trailing comma
                if let Some(Token::RBrace) = tokens.peek() {
                    continue;
                }
            }
            Token::Colon => {
                if current_key.is_none() || expect_value {
                     return Err(SchemeError::Parser("Unexpected colon in map literal".to_string()));
                 }
                tokens.next(); // Consume ':'
                expect_value = true;
            }
             Token::Symbol(key_str) => {
                if expect_value { // Parsing the value part
                    let value_expr = parse_expr(tokens)?;
                     let key = current_key.take().unwrap(); // Should be Some if expect_value is true
                    map.insert(key, value_expr);
                    expect_value = false;
                    expect_comma = true; // Expect comma after value (or closing brace)
                } else if current_key.is_some() {
                    return Err(SchemeError::Parser("Expected ':' after map key".to_string()));
                 }
                else { // Parsing the key part
                     if expect_comma {
                          return Err(SchemeError::Parser("Expected comma before next key in map literal".to_string()));
                     }
                    current_key = Some(key_str.clone());
                    tokens.next(); // Consume symbol token
                }
            }
             // Allow string keys? For now, stick to symbols as keys like JS allows unquoted keys
            // Token::String(key_str) => { ... }

            _ => { // Any other token is either a value or an error
                 if expect_value { // Parsing the value part
                    let value_expr = parse_expr(tokens)?;
                    let key = current_key.take().unwrap();
                    map.insert(key, value_expr);
                    expect_value = false;
                    expect_comma = true;
                 } else if current_key.is_some() {
                     return Err(SchemeError::Parser(format!("Expected ':' after map key '{}'", current_key.unwrap())));
                 } else {
                     return Err(SchemeError::Parser(format!("Unexpected token {:?} in map literal; expected key (symbol)", "token")));
                 }
            }
        }
    }

    Err(SchemeError::Parser("Unmatched '{'".to_string()))
}


pub fn parse(input: &str) -> Result<Value> {
    let tokens = tokenize(input)?;
    if tokens.is_empty() {
        // Special case for empty input or only whitespace/comments
        return Ok(Value::Symbol("".to_string())); // Return an inert value or a specific marker?
                                                  // Let's use an empty symbol for now, eval can ignore it.
    }
    let mut token_iter = tokens.iter().peekable();
    let result = parse_expr(&mut token_iter)?;

    // Ensure all tokens were consumed
    if token_iter.peek().is_some() {
        Err(SchemeError::Parser("Unexpected tokens after expression".to_string()))
    } else {
        Ok(result)
    }
}