use std::{env, fs, path::Path};

#[derive(Debug, PartialEq)]
enum Tokens {
    LeftBrace,
    RightBrace,
    DoubleQuote,
    // Key(String),
    Colon,
    StringValue(String),
    EOF,
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
            '"' => tokens.push(Tokens::DoubleQuote),
            ':' => tokens.push(Tokens::Colon),
            _ => {
                if c.is_alphanumeric() {
                    let mut buffer: String = c.to_string();
                    while let Some(c) = chars.peek() {
                        if *c == '{' || *c == '}' || *c == '"' || *c == ':' {
                            tokens.push(Tokens::StringValue(buffer));
                            break;
                        }
                        let c = match chars.next() {
                            Some(c) => c,
                            None => break,
                        };
                        buffer.push(c);
                    }
                } else {
                    return Err(ParserErrors::TokenizeError);
                }
            }
        }
    }
    tokens.push(Tokens::EOF);
    Ok(tokens)
}

enum JSONStruct {
    Object(Object),
    List,
    Value,
}
struct Object {
    name: String,
    value: String, //keeping it string for now
                   // value: json_struct, this allows nesting in theory
}
struct JSONDocument {
    data: Vec<JSONStruct>,
}
impl JSONDocument {
    fn parse(&mut self, tokens: Vec<Tokens>) -> Result<bool, ParserErrors> {
        //initially let's parse objects
        if tokens.len() < 2 {
            return Err(ParserErrors::ParsingError(
                "not enough elements".to_string(),
            ));
        }
        let mut tokens = tokens.iter().peekable();
        while let Some(token) = tokens.next() {
            match token {
                Tokens::LeftBrace => {
                    //parse object
                    let mut object_name = String::new();
                    let mut object_value = String::new();

                    //quotes or (closing) right brace
                    let Some(token) = tokens.next() else { todo!() };
                    match token {
                        Tokens::DoubleQuote => (),
                        Tokens::RightBrace => continue, //empty object
                        _ => {
                            return Err(ParserErrors::ParsingError(
                                "expected double quote".to_string(),
                            ))
                        }
                    };

                    //object name
                    let Some(token) = tokens.next() else { todo!() };
                    match token {
                        Tokens::StringValue(name) => object_name = name.clone(),
                        _ => {
                            return Err(ParserErrors::ParsingError(
                                "expected object name".to_string(),
                            ))
                        }
                    };

                    //quotes
                    let Some(token) = tokens.next() else { todo!() };
                    match token {
                        Tokens::DoubleQuote => (),
                        _ => {
                            return Err(ParserErrors::ParsingError(
                                "expected double quote".to_string(),
                            ))
                        }
                    };

                    //colon
                    let Some(token) = tokens.next() else { todo!() };
                    match token {
                        Tokens::Colon => (),
                        _ => return Err(ParserErrors::ParsingError("expected colon".to_string())),
                    };

                    //quotes
                    let Some(token) = tokens.next() else { todo!() };
                    match token {
                        Tokens::DoubleQuote => (),
                        _ => {
                            return Err(ParserErrors::ParsingError(
                                "expected double quote".to_string(),
                            ))
                        }
                    };

                    let Some(token) = tokens.next() else { todo!() };
                    match token {
                        Tokens::StringValue(value) => object_value = value.clone(),
                        // Tokens::Numerical
                        // Tokens::LeftBrace -- object
                        // Tokens::LeftBracket -- array
                        // Tokens::Boolean true or false
                        // Tokens::null
                        _ => {
                            return Err(ParserErrors::ParsingError(
                                "expected object value".to_string(),
                            ))
                        }
                    };

                    //quotes
                    let Some(token) = tokens.next() else { todo!() };
                    match token {
                        Tokens::DoubleQuote => (),
                        _ => {
                            return Err(ParserErrors::ParsingError(
                                "expected double quote".to_string(),
                            ))
                        }
                    };
                    let object = Object {
                        name: object_name,
                        value: object_value,
                    };
                    self.data.push(JSONStruct::Object(object));

                    let Some(token) = tokens.next() else { todo!() };
                    match token {
                        Tokens::RightBrace => (),
                        _ => {
                            return Err(ParserErrors::ParsingError(
                                "expected right brace".to_string(),
                            ))
                        }
                    };
                }
                // Tokens::LeftBracket => todo!(),//parse array
                _ => {
                    return Err(ParserErrors::ParsingError(
                        "expected object or list".to_string(),
                    ))
                }
            };
        }
        Ok(true)
    }
}
// fn parse(tokens: Vec<Tokens>) -> bool {
//     if tokens.len() < 2 {
//         return false;
//     }
//     // if tokens[0] != Tokens::LeftBrace || tokens[1] != Tokens::RightBrace {
//     if tokens[0] != Tokens::LeftBrace {
//         return false;
//     }
//     let mut tokens = tokens.iter().peekable();
//     while let Some(token) = tokens.next() {
//         match token {}
//     }
//     true
// }

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

    let json_document = &mut JSONDocument { data: Vec::new() };
    json_document.parse(tokens)?;

    println!("valid json!");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tokenize_on_braces() {
        let tokens = tokenize("{}".into()).unwrap();

        assert_eq!(Tokens::LeftBrace, tokens[0]);

        assert_eq!(Tokens::RightBrace, tokens[1]);
    }
    #[test]
    fn test_tokenize_string_key_values() {
        //{"key": "value"}
        let tokens = tokenize("{\"key\": \"value\"}".into()).unwrap();
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

    //i think it shouldn't error on unkown chars that is the job of the
    // parser
    #[test]
    fn test_tokenize_errors_on_unkown() {}

    #[test]
    fn test_parse_works_on_single_braces_document() {
        let tokens: Vec<Tokens> = vec![Tokens::LeftBrace, Tokens::RightBrace];
        let json_document = &mut JSONDocument { data: Vec::new() };
        let valid = json_document.parse(tokens).unwrap();

        assert!(valid);
    }

    #[test]
    fn test_parse_fails_on_invalid_tokens() {
        let tokens: Vec<Tokens> = vec![Tokens::RightBrace, Tokens::RightBrace];

        let json_document = &mut JSONDocument { data: Vec::new() };

        assert!(json_document.parse(tokens).is_err());
    }

    #[test]
    fn test_parse_fails_on_short_tokens() {
        let tokens: Vec<Tokens> = Vec::new();
        let json_document = &mut JSONDocument { data: Vec::new() };
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
        ];

        let json_document = &mut JSONDocument { data: Vec::new() };
        let valid = json_document.parse(tokens).unwrap();
        assert!(valid);
    }
}
