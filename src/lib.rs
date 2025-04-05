use std::{env, fs, iter::Peekable, path::Path};

#[derive(Debug, PartialEq, Clone)]
enum Tokens {
    LeftBrace,
    RightBrace,
    LeftBracket,
    RightBracket,
    DoubleQuote,
    Colon,
    Comma,
    StringValue(String),
    BooleanValue(bool),
    IntegerValue(i32),
    FloatValue(f32),
    NullValue,
    EOF,
}
impl Tokens {
    fn parse_string_value<'a>(
        tokens: &mut impl Iterator<Item = &'a Tokens>,
    ) -> Result<String, ParserErrors> {
        //the idea is to make sure the correct structure("<value>") exist and
        //return the StringValue token already parsed in a subset of items

        let Some(token) = tokens.next() else {
            return Err(ParserErrors::ParsingError(
                "expected more tokens".to_string(),
            ));
        };

        let name = match token {
            Tokens::StringValue(name) => name,
            _ => {
                return Err(ParserErrors::ParsingError(
                    "expected String Value".to_string(),
                ))
            }
        };
        //quotes
        let Some(token) = tokens.next() else {
            return Err(ParserErrors::ParsingError(
                "expected more tokens".to_string(),
            ));
        };
        if *token != Tokens::DoubleQuote {
            return Err(ParserErrors::ParsingError(
                "expected double quote(\")".to_string(),
            ));
        }
        Ok(name.clone())
    }
}

fn tokenize(input: String) -> Result<Vec<Tokens>, ParserErrors> {
    let mut tokens: Vec<Tokens> = Vec::new();
    let mut chars = input.chars().peekable();
    while let Some(c) = chars.next() {
        //eat the whitespace nom nom nom
        if c.is_whitespace() {
            continue;
        }

        match c {
            '{' => tokens.push(Tokens::LeftBrace),
            '}' => tokens.push(Tokens::RightBrace),
            '[' => tokens.push(Tokens::LeftBracket),
            ']' => tokens.push(Tokens::RightBracket),
            '"' => {
                tokens.push(Tokens::DoubleQuote);
                let mut buffer: String = String::new();
                while let Some(c) = chars.next() {
                    //don't eat the whitespace inside quotes
                    if c == '"' {
                        let Some(peek_c) = chars.peek() else {
                            return Err(ParserErrors::TokenizeError);
                        };
                        if *peek_c != ',' && *peek_c != ':' && *peek_c != ']' && *peek_c != '}' {
                            //this is a escaped double quote
                            buffer.push(c);
                            continue;
                        }
                        tokens.push(Tokens::StringValue(buffer));
                        tokens.push(Tokens::DoubleQuote);
                        break;
                    }
                    buffer.push(c);
                }
            }
            ':' => tokens.push(Tokens::Colon),
            ',' => tokens.push(Tokens::Comma),
            _ => {
                if c.is_alphanumeric() || c == '-' || c == '+' {
                    let mut buffer: String = c.to_string();

                    while let Some(c) = chars.peek() {
                        if !c.is_alphanumeric() && *c != '.' && *c != '-' && *c != '+' {
                            if buffer == "true" {
                                tokens.push(Tokens::BooleanValue(true));
                            } else if buffer == "false" {
                                tokens.push(Tokens::BooleanValue(false));
                            } else if buffer == "null" {
                                tokens.push(Tokens::NullValue);
                            } else if buffer.contains(".")
                                || buffer.contains("e")
                                || buffer.contains("E")
                            {
                                println!("{buffer}");
                                let float = buffer.parse::<f32>()?;
                                tokens.push(Tokens::FloatValue(float));
                            } else {
                                let integer = buffer.parse::<i32>()?;
                                tokens.push(Tokens::IntegerValue(integer));
                            }
                            break;
                        }
                        let c = match chars.next() {
                            Some(c) => c,
                            None => break,
                        };
                        buffer.push(c);
                    }
                } else {
                    println!("tokenix {:?}", tokens);
                    return Err(ParserErrors::TokenizeError);
                }
            }
        }
    }
    tokens.push(Tokens::EOF);
    Ok(tokens)
}

fn parse_object<'a>(
    tokens: &mut Peekable<impl Iterator<Item = &'a Tokens>>,
) -> Result<(), ParserErrors> {
    while let Some(token) = tokens.next() {
        match token {
            Tokens::EOF => break,
            Tokens::LeftBrace => continue,
            Tokens::RightBrace => {
                // empty object
                //Check if next value is not { means no more objects
                // continue;
                break;
            }
            Tokens::Comma => continue,
            _ => {
                if *token != Tokens::DoubleQuote {
                    return Err(ParserErrors::ParsingError(
                        "expected double quote(\")".to_string(),
                    ));
                }
                //todo skip { if new iteration the function enters with the { already consumed
                // let key =Tokens::parse_string_value(tokens)?;
                Tokens::parse_string_value(tokens)?;

                let Some(token) = tokens.next() else {
                    return Err(ParserErrors::ParsingError(
                        "expected more token".to_string(),
                    ));
                };
                if *token != Tokens::Colon {
                    return Err(ParserErrors::ParsingError(
                        "expected colon token".to_string(),
                    ));
                }

                let Some(token) = tokens.peek() else {
                    return Err(ParserErrors::ParsingError("expected token".to_string()));
                };

                match *token {
                    Tokens::LeftBrace => {
                        parse_object(tokens)?;
                    }
                    Tokens::LeftBracket => parse_list(tokens)?,
                    Tokens::DoubleQuote => {
                        let Some(_) = tokens.next() else {
                            return Err(ParserErrors::ParsingError(
                                "expected more tokens".to_string(),
                            ));
                        };
                        let Some(token) = tokens.next() else {
                            return Err(ParserErrors::ParsingError(
                                "expected more tokens".to_string(),
                            ));
                        };
                        parse_value(token)?;
                        let Some(token) = tokens.next() else {
                            return Err(ParserErrors::ParsingError(
                                "expected \" double quote".to_string(),
                            ));
                        };
                        if *token != Tokens::DoubleQuote {
                            return Err(ParserErrors::ParsingError(
                                "expected \" double quote".to_string(),
                            ));
                        }
                    }
                    _ => {
                        let Some(token) = tokens.next() else {
                            return Err(ParserErrors::ParsingError(
                                "expected more tokens".to_string(),
                            ));
                        };
                        parse_value(token)?;
                    }
                };
            }
        }
    }
    Ok(())
}

fn parse_list<'a>(
    tokens: &mut Peekable<impl Iterator<Item = &'a Tokens>>,
) -> Result<(), ParserErrors> {
    while let Some(token) = tokens.next() {
        match token {
            Tokens::EOF => break,
            Tokens::LeftBracket => continue,
            Tokens::RightBracket =>
            //empty list
            {
                // continue;
                break;
            }
            Tokens::Comma => continue,
            Tokens::LeftBrace => parse_object(tokens)?,
            Tokens::DoubleQuote => {
                let Some(token) = tokens.next() else {
                    return Err(ParserErrors::ParsingError(
                        "expected more tokens".to_string(),
                    ));
                };
                parse_value(token)?;
                let Some(token) = tokens.next() else {
                    return Err(ParserErrors::ParsingError(
                        "expected \" double quote".to_string(),
                    ));
                };
                if *token != Tokens::DoubleQuote {
                    return Err(ParserErrors::ParsingError(
                        "expected \" double quote".to_string(),
                    ));
                }
            }
            _ => parse_value(token)?,
        }
    }
    Ok(())
}

fn parse_value(token: &Tokens) -> Result<(), ParserErrors> {
    match token {
        Tokens::StringValue(_) => (),
        Tokens::IntegerValue(_) => (),
        Tokens::FloatValue(_) => (),
        Tokens::BooleanValue(_) => (),
        Tokens::NullValue => (),
        _ => return Err(ParserErrors::ParsingError("unexpected token".to_string())),
    };
    Ok(())
}

struct JSONDocument {}
impl JSONDocument {
    fn parse(&mut self, tokens: Vec<Tokens>) -> Result<bool, ParserErrors> {
        if tokens.len() < 2 {
            return Err(ParserErrors::ParsingError(
                "not enough elements".to_string(),
            ));
        }

        let mut tokens = tokens.iter().peekable();

        let Some(token) = tokens.peek() else {
            return Err(ParserErrors::ParsingError("expected token".to_string()));
        };
        match token {
            Tokens::LeftBrace => parse_object(&mut tokens)?,
            Tokens::LeftBracket => parse_list(&mut tokens)?,
            _ => {
                return Err(ParserErrors::ParsingError(
                    "invalid inital token".to_string(),
                ))
            }
        };

        Ok(true)
    }
}

struct Config {
    file_path: String,
}

#[derive(Debug, thiserror::Error)]
pub enum ParserErrors {
    #[error("{0}")]
    ArgumentError(String),
    #[error("not able to tokenize")]
    TokenizeError,
    #[error("invalid json: {0}")]
    ParsingError(String),
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("Parse Int Error: {0}")]
    ParseIntError(#[from] std::num::ParseIntError),
    #[error("Parse Float Error: {0}")]
    ParseFloatError(#[from] std::num::ParseFloatError),
}

impl Config {
    fn build(args: Vec<String>) -> Result<Config, ParserErrors> {
        if args.len() <= 1 {
            return Err(ParserErrors::ArgumentError(
                "please provide a filename".to_string(),
            ));
        } else if args.len() > 2 {
            return Err(ParserErrors::ArgumentError(
                "too many arguments".to_string(),
            ));
        }
        Ok(Config {
            file_path: args[1].clone(),
        })
    }
}

pub fn run() -> Result<(), ParserErrors> {
    let config = Config::build(env::args().collect())?;

    let path = Path::new(config.file_path.as_str());
    let data = match fs::read_to_string(path) {
        Ok(data) => data,
        Err(e) => return Err(ParserErrors::IoError(e)),
    };

    let tokens = tokenize(data)?;

    let json_document = &mut JSONDocument {};
    json_document.parse(tokens)?;

    println!("valid json!");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    //tokenize tests
    #[test]
    fn test_tokenize_on_braces() {
        let tokens = tokenize("{}".into()).unwrap();

        assert_eq!(tokens.len(), 3);
        assert_eq!(Tokens::LeftBrace, tokens[0]);
        assert_eq!(Tokens::RightBrace, tokens[1]);
    }
    #[test]
    fn test_tokenize_string_values() {
        //{"key": "value"}
        let tokens = tokenize("{\"key\": \"value\"}".into()).unwrap();
        assert_eq!(tokens.len(), 10);
        assert_eq!(Tokens::LeftBrace, tokens[0]);
        assert_eq!(Tokens::DoubleQuote, tokens[1]);
        assert_eq!(Tokens::StringValue("key".into()), tokens[2]);
        assert_eq!(Tokens::DoubleQuote, tokens[3]);
        assert_eq!(Tokens::Colon, tokens[4]);
        assert_eq!(Tokens::DoubleQuote, tokens[5]);
        assert_eq!(Tokens::StringValue("value".into()), tokens[6]);
        assert_eq!(Tokens::DoubleQuote, tokens[7]);
        assert_eq!(Tokens::RightBrace, tokens[8]);
    }

    #[test]
    fn test_tokenize_errors_on_unknown() {
        assert!(tokenize("?".into()).is_err());
    }

    #[test]
    fn test_tokenize_bool_values() {
        let tokens = tokenize("{\"key\": true, \"key2\": false}".into()).unwrap();
        assert_eq!(tokens.len(), 14);
        assert_eq!(Tokens::LeftBrace, tokens[0]);
        assert_eq!(Tokens::DoubleQuote, tokens[1]);
        assert_eq!(Tokens::StringValue("key".into()), tokens[2]);
        assert_eq!(Tokens::DoubleQuote, tokens[3]);
        assert_eq!(Tokens::Colon, tokens[4]);
        assert_eq!(Tokens::BooleanValue(true), tokens[5]);
        assert_eq!(Tokens::Comma, tokens[6]);
        assert_eq!(Tokens::DoubleQuote, tokens[7]);
        assert_eq!(Tokens::StringValue("key2".into()), tokens[8]);
        assert_eq!(Tokens::DoubleQuote, tokens[9]);
        assert_eq!(Tokens::Colon, tokens[10]);
        assert_eq!(Tokens::BooleanValue(false), tokens[11]);
        assert_eq!(Tokens::RightBrace, tokens[12]);
    }

    #[test]
    fn test_tokenize_integer_values() {
        let tokens = tokenize("{\"key\": -32, \"key2\": 14}".into()).unwrap();
        assert_eq!(tokens.len(), 14);
        assert_eq!(Tokens::LeftBrace, tokens[0]);
        assert_eq!(Tokens::DoubleQuote, tokens[1]);
        assert_eq!(Tokens::StringValue("key".into()), tokens[2]);
        assert_eq!(Tokens::DoubleQuote, tokens[3]);
        assert_eq!(Tokens::Colon, tokens[4]);
        assert_eq!(Tokens::IntegerValue(-32), tokens[5]);
        assert_eq!(Tokens::Comma, tokens[6]);
        assert_eq!(Tokens::DoubleQuote, tokens[7]);
        assert_eq!(Tokens::StringValue("key2".into()), tokens[8]);
        assert_eq!(Tokens::DoubleQuote, tokens[9]);
        assert_eq!(Tokens::Colon, tokens[10]);
        assert_eq!(Tokens::IntegerValue(14), tokens[11]);
        assert_eq!(Tokens::RightBrace, tokens[12]);
    }

    #[test]
    fn test_tokenize_float_values() {
        let tokens = tokenize("{\"key\": -3.2, \"key2\": 0.14}".into()).unwrap();
        assert_eq!(tokens.len(), 14);
        assert_eq!(Tokens::LeftBrace, tokens[0]);
        assert_eq!(Tokens::DoubleQuote, tokens[1]);
        assert_eq!(Tokens::StringValue("key".into()), tokens[2]);
        assert_eq!(Tokens::DoubleQuote, tokens[3]);
        assert_eq!(Tokens::Colon, tokens[4]);
        assert_eq!(Tokens::FloatValue(-3.2), tokens[5]);
        assert_eq!(Tokens::Comma, tokens[6]);
        assert_eq!(Tokens::DoubleQuote, tokens[7]);
        assert_eq!(Tokens::StringValue("key2".into()), tokens[8]);
        assert_eq!(Tokens::DoubleQuote, tokens[9]);
        assert_eq!(Tokens::Colon, tokens[10]);
        assert_eq!(Tokens::FloatValue(0.14), tokens[11]);
        assert_eq!(Tokens::RightBrace, tokens[12]);
    }

    #[test]
    fn test_tokenize_float_scientific_notation_values() {
        let tokens = tokenize("{\"key\": -3E3, \"key2\": 14E-4}".into()).unwrap();
        //not parsing negative exponent e-1
        assert_eq!(tokens.len(), 14);
        assert_eq!(Tokens::LeftBrace, tokens[0]);
        assert_eq!(Tokens::DoubleQuote, tokens[1]);
        assert_eq!(Tokens::StringValue("key".into()), tokens[2]);
        assert_eq!(Tokens::DoubleQuote, tokens[3]);
        assert_eq!(Tokens::Colon, tokens[4]);
        assert_eq!(Tokens::FloatValue(-3000.0), tokens[5]);
        assert_eq!(Tokens::Comma, tokens[6]);
        assert_eq!(Tokens::DoubleQuote, tokens[7]);
        assert_eq!(Tokens::StringValue("key2".into()), tokens[8]);
        assert_eq!(Tokens::DoubleQuote, tokens[9]);
        assert_eq!(Tokens::Colon, tokens[10]);
        assert_eq!(Tokens::FloatValue(0.0014), tokens[11]);
        assert_eq!(Tokens::RightBrace, tokens[12]);
    }

    #[test]
    fn test_tokenize_null_values() {
        let tokens = tokenize("{\"key\": null, \"key2\": null}".into()).unwrap();
        assert_eq!(tokens.len(), 14);
        assert_eq!(Tokens::LeftBrace, tokens[0]);
        assert_eq!(Tokens::DoubleQuote, tokens[1]);
        assert_eq!(Tokens::StringValue("key".into()), tokens[2]);
        assert_eq!(Tokens::DoubleQuote, tokens[3]);
        assert_eq!(Tokens::Colon, tokens[4]);
        assert_eq!(Tokens::NullValue, tokens[5]);
        assert_eq!(Tokens::Comma, tokens[6]);
        assert_eq!(Tokens::DoubleQuote, tokens[7]);
        assert_eq!(Tokens::StringValue("key2".into()), tokens[8]);
        assert_eq!(Tokens::DoubleQuote, tokens[9]);
        assert_eq!(Tokens::Colon, tokens[10]);
        assert_eq!(Tokens::NullValue, tokens[11]);
        assert_eq!(Tokens::RightBrace, tokens[12]);
    }

    #[test]
    fn test_tokenize_on_brackets() {
        let tokens = tokenize("[]".into()).unwrap();

        assert_eq!(tokens.len(), 3);
        assert_eq!(Tokens::LeftBracket, tokens[0]);
        assert_eq!(Tokens::RightBracket, tokens[1]);
    }

    #[test]
    fn test_tokenize_list() {
        let tokens = tokenize("[\"one\", 2, true]".into()).unwrap();

        assert_eq!(tokens.len(), 10);
        assert_eq!(Tokens::LeftBracket, tokens[0]);
        assert_eq!(Tokens::DoubleQuote, tokens[1]);
        assert_eq!(Tokens::StringValue("one".into()), tokens[2]);
        assert_eq!(Tokens::DoubleQuote, tokens[3]);
        assert_eq!(Tokens::Comma, tokens[4]);
        assert_eq!(Tokens::IntegerValue(2), tokens[5]);
        assert_eq!(Tokens::Comma, tokens[6]);
        assert_eq!(Tokens::BooleanValue(true), tokens[7]);
        assert_eq!(Tokens::RightBracket, tokens[8]);
    }

    #[test]
    fn test_tokenize_list_nested_object() {
        let tokens = tokenize("[\"one\", 2, { \"inner key\": true}]".into()).unwrap();

        assert_eq!(tokens.len(), 16);
        assert_eq!(Tokens::LeftBracket, tokens[0]);
        assert_eq!(Tokens::DoubleQuote, tokens[1]);
        assert_eq!(Tokens::StringValue("one".into()), tokens[2]);
        assert_eq!(Tokens::DoubleQuote, tokens[3]);
        assert_eq!(Tokens::Comma, tokens[4]);
        assert_eq!(Tokens::IntegerValue(2), tokens[5]);
        assert_eq!(Tokens::Comma, tokens[6]);

        assert_eq!(Tokens::LeftBrace, tokens[7]);
        assert_eq!(Tokens::DoubleQuote, tokens[8]);
        assert_eq!(Tokens::StringValue("inner key".into()), tokens[9]);
        assert_eq!(Tokens::DoubleQuote, tokens[10]);
        assert_eq!(Tokens::Colon, tokens[11]);
        assert_eq!(Tokens::BooleanValue(true), tokens[12]);
        assert_eq!(Tokens::RightBrace, tokens[13]);
        assert_eq!(Tokens::RightBracket, tokens[14]);
    }

    #[test]
    fn test_tokenize_quote() {
        let input: String = "{\"key\": \"\"\"}".into();

        let tokens = tokenize(input).unwrap();

        assert_eq!(tokens.len(), 10);
        assert_eq!(Tokens::LeftBrace, tokens[0]);
        assert_eq!(Tokens::DoubleQuote, tokens[1]);
        assert_eq!(Tokens::StringValue("key".into()), tokens[2]);
        assert_eq!(Tokens::DoubleQuote, tokens[3]);
        assert_eq!(Tokens::Colon, tokens[4]);
        assert_eq!(Tokens::DoubleQuote, tokens[5]);
        assert_eq!(Tokens::StringValue("\"".into()), tokens[6]);
        assert_eq!(Tokens::DoubleQuote, tokens[7]);
        assert_eq!(Tokens::RightBrace, tokens[8]);
    }

    //Parsing tests
    //------------------
    #[test]
    fn test_parse_works_on_single_braces_document() {
        let tokens: Vec<Tokens> = vec![Tokens::LeftBrace, Tokens::RightBrace];
        let json_document = &mut JSONDocument {};
        let valid = json_document.parse(tokens).unwrap();

        assert!(valid);
    }

    #[test]
    fn test_parse_fails_on_invalid_tokens() {
        let tokens: Vec<Tokens> = vec![Tokens::RightBrace, Tokens::RightBrace];

        let json_document = &mut JSONDocument {};

        assert!(json_document.parse(tokens).is_err());
    }

    #[test]
    fn test_parse_fails_on_short_tokens() {
        let tokens: Vec<Tokens> = Vec::new();
        let json_document = &mut JSONDocument {};
        assert!(json_document.parse(tokens).is_err());
    }

    #[test]
    fn test_parse_key_value_tokens_valid() {
        let tokens = vec![
            Tokens::LeftBrace,
            Tokens::DoubleQuote,
            Tokens::StringValue("key".into()),
            Tokens::DoubleQuote,
            Tokens::Colon,
            Tokens::DoubleQuote,
            Tokens::StringValue("value".into()),
            Tokens::DoubleQuote,
            Tokens::RightBrace,
            Tokens::EOF,
        ];

        let json_document = &mut JSONDocument {};
        let valid = json_document.parse(tokens).unwrap();
        assert!(valid);
    }

    #[test]
    fn test_parse_boolean_integer_float_null_value_valid() {
        let tokens = vec![
            //{"key":true,
            Tokens::LeftBrace,
            Tokens::DoubleQuote,
            Tokens::StringValue("key".into()),
            Tokens::DoubleQuote,
            Tokens::Colon,
            Tokens::BooleanValue(true),
            Tokens::Comma,
            //"key":42,
            Tokens::DoubleQuote,
            Tokens::StringValue("key".into()),
            Tokens::DoubleQuote,
            Tokens::Colon,
            Tokens::IntegerValue(42),
            Tokens::Comma,
            //"key":3.2,
            Tokens::DoubleQuote,
            Tokens::StringValue("key".into()),
            Tokens::DoubleQuote,
            Tokens::Colon,
            Tokens::FloatValue(-3.2),
            Tokens::Comma,
            //"key":null}
            Tokens::DoubleQuote,
            Tokens::StringValue("key".into()),
            Tokens::DoubleQuote,
            Tokens::Colon,
            Tokens::NullValue,
            Tokens::RightBrace,
            Tokens::EOF,
        ];

        let json_document = &mut JSONDocument {};
        let valid = json_document.parse(tokens).unwrap();
        assert!(valid);
    }
    #[test]
    fn test_parse_works_on_single_brackets_document() {
        let tokens: Vec<Tokens> = vec![Tokens::LeftBracket, Tokens::RightBracket, Tokens::EOF];
        let json_document = &mut JSONDocument {};
        let valid = json_document.parse(tokens).unwrap();

        assert!(valid);
    }

    #[test]
    fn test_parse_list() {
        let tokens = vec![
            Tokens::LeftBracket,
            Tokens::DoubleQuote,
            Tokens::StringValue("one".into()),
            Tokens::DoubleQuote,
            Tokens::Comma,
            Tokens::IntegerValue(42),
            Tokens::RightBracket,
            Tokens::EOF,
        ];
        let json_document = &mut JSONDocument {};
        let valid = json_document.parse(tokens).unwrap();

        assert!(valid);
    }

    #[test]
    fn test_parse_object_and_list() {
        let tokens = vec![
            Tokens::LeftBrace,
            Tokens::DoubleQuote,
            Tokens::StringValue("key".into()),
            Tokens::DoubleQuote,
            Tokens::Colon,
            Tokens::DoubleQuote,
            Tokens::StringValue("value".into()),
            Tokens::DoubleQuote,
            Tokens::Comma,
            Tokens::DoubleQuote,
            Tokens::StringValue("key-n".into()),
            Tokens::DoubleQuote,
            Tokens::Colon,
            Tokens::IntegerValue(101),
            Tokens::Comma,
            Tokens::DoubleQuote,
            Tokens::StringValue("key-o".into()),
            Tokens::DoubleQuote,
            Tokens::Colon,
            Tokens::LeftBrace,
            Tokens::RightBrace,
            Tokens::Comma,
            Tokens::DoubleQuote,
            Tokens::StringValue("key-l".into()),
            Tokens::DoubleQuote,
            Tokens::Colon,
            Tokens::LeftBracket,
            Tokens::RightBracket,
            Tokens::RightBrace,
            Tokens::EOF,
        ];

        let json_document = &mut JSONDocument {};
        let valid = json_document.parse(tokens).unwrap();
        assert!(valid);
    }

    #[test]
    fn test_parse_object_and_list_nested() {
        let tokens = vec![
            Tokens::LeftBrace,
            Tokens::DoubleQuote,
            Tokens::StringValue("key".into()),
            Tokens::DoubleQuote,
            Tokens::Colon,
            Tokens::DoubleQuote,
            Tokens::StringValue("value".into()),
            Tokens::DoubleQuote,
            Tokens::Comma,
            Tokens::DoubleQuote,
            Tokens::StringValue("key-n".into()),
            Tokens::DoubleQuote,
            Tokens::Colon,
            Tokens::IntegerValue(101),
            Tokens::Comma,
            Tokens::DoubleQuote,
            Tokens::StringValue("key-o".into()),
            Tokens::DoubleQuote,
            Tokens::Colon,
            Tokens::LeftBrace,
            Tokens::DoubleQuote,
            Tokens::StringValue("inner key".into()),
            Tokens::DoubleQuote,
            Tokens::Colon,
            Tokens::DoubleQuote,
            Tokens::StringValue("inner value".into()),
            Tokens::DoubleQuote,
            Tokens::RightBrace,
            Tokens::Comma,
            Tokens::DoubleQuote,
            Tokens::StringValue("key-l".into()),
            Tokens::DoubleQuote,
            Tokens::Colon,
            Tokens::LeftBracket,
            Tokens::DoubleQuote,
            Tokens::StringValue("list value".into()),
            Tokens::DoubleQuote,
            Tokens::RightBracket,
            Tokens::RightBrace,
            Tokens::EOF,
        ];

        let json_document = &mut JSONDocument {};
        let valid = json_document.parse(tokens).unwrap();
        assert!(valid);
    }
}
