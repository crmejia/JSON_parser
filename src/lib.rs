use std::{env, fs, path::Path};

#[derive(Debug, PartialEq, Clone)]
enum Tokens {
    LeftBrace,
    RightBrace,
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
            ',' => tokens.push(Tokens::Comma),
            _ => {
                if c.is_alphanumeric() || c == '-' {
                    let mut buffer: String = c.to_string();

                    while let Some(c) = chars.peek() {
                        // if *c == '{' || *c == '}' || *c == '"' || *c == ':' || *c == ',' {
                        if !c.is_alphanumeric() && *c != '.' && *c != '-' {
                            if *c != '"'
                                && (buffer == "true" || buffer == "false" || buffer == "null")
                            {
                                if buffer == "true" {
                                    tokens.push(Tokens::BooleanValue(true));
                                } else if buffer == "false" {
                                    tokens.push(Tokens::BooleanValue(false));
                                } else {
                                    tokens.push(Tokens::NullValue);
                                }
                            } else if *c == '"' {
                                tokens.push(Tokens::StringValue(buffer));
                            } else {
                                //parse a numerical value
                                if buffer.contains(".")
                                    || buffer.contains("e")
                                    || buffer.contains("E")
                                {
                                    let float = buffer.parse::<f32>()?;
                                    tokens.push(Tokens::FloatValue(float));
                                } else {
                                    let integer = buffer.parse::<i32>()?;
                                    tokens.push(Tokens::IntegerValue(integer));
                                }
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
    // Value,
}
struct Object {
    name: String,
    value: Tokens, //keeping it string for now
}
struct JSONDocument {
    data: Vec<JSONStruct>,
}
impl JSONDocument {
    fn parse(&mut self, tokens: Vec<Tokens>) -> Result<bool, ParserErrors> {
        if tokens.len() < 2 {
            return Err(ParserErrors::ParsingError(
                "not enough elements".to_string(),
            ));
        }
        let mut tokens = tokens.iter().peekable();
        while let Some(token) = tokens.next() {
            match token {
                Tokens::LeftBrace => {
                    let Some(token) = tokens.peek() else {
                        return Err(ParserErrors::ParsingError("expected token".to_string()));
                    };
                    if **(token) == Tokens::RightBrace {
                        //empty object. Note that it is valid and should be pushed into the document
                        self.data.push(JSONStruct::Object(Object {
                            name: "".to_string(),
                            value: Tokens::StringValue("".into()),
                        }));
                        tokens.next();
                        continue;
                    }
                    while let Some(token) = tokens.next() {
                        //parse object
                        let mut object_name = String::new();
                        let mut object_value = Tokens::EOF;

                        //comma or quotes
                        match token {
                            Tokens::Comma => {
                                let Some(token) = tokens.next() else {
                                    return Err(ParserErrors::ParsingError(
                                        "expected token".to_string(),
                                    ));
                                };
                                if *token != Tokens::DoubleQuote {
                                    return Err(ParserErrors::ParsingError(
                                        "expected double quote".to_string(),
                                    ));
                                }
                            }
                            Tokens::DoubleQuote => (),
                            Tokens::EOF => break,
                            _ => {
                                return Err(ParserErrors::ParsingError(
                                    "expected double quote".to_string(),
                                ));
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
                            _ => {
                                return Err(ParserErrors::ParsingError(
                                    "expected colon".to_string(),
                                ))
                            }
                        };

                        //value
                        let Some(token) = tokens.next() else { todo!() };
                        match token {
                            Tokens::DoubleQuote => {
                                //String Value
                                let Some(token) = tokens.next() else { todo!() };
                                match token {
                                    Tokens::StringValue(_) => object_value = token.clone(),
                                    _ => {
                                        return Err(ParserErrors::ParsingError(
                                            "expected object value".to_string(),
                                        ))
                                    }
                                };
                                // closing quotes
                                let Some(token) = tokens.next() else { todo!() };
                                match token {
                                    Tokens::DoubleQuote => (),
                                    _ => {
                                        return Err(ParserErrors::ParsingError(
                                            "expected double quote".to_string(),
                                        ))
                                    }
                                };
                            }
                            Tokens::IntegerValue(_) => object_value = token.clone(),
                            Tokens::FloatValue(_) => object_value = token.clone(),
                            Tokens::BooleanValue(_) => object_value = token.clone(),
                            Tokens::NullValue => object_value = token.clone(),
                            // Tokens::LeftBrace -- object
                            // Tokens::LeftBracket -- array
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
                            Tokens::Comma => continue,
                            _ => {
                                return Err(ParserErrors::ParsingError(
                                    "expected right brace".to_string(),
                                ))
                            }
                        };
                    }
                }
                // Tokens::LeftBracket => todo!(),//parse array
                Tokens::EOF => break,
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

    let json_document = &mut JSONDocument { data: Vec::new() };
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

    //Parsing tests
    //------------------
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
            Tokens::EOF,
        ];

        let json_document = &mut JSONDocument { data: Vec::new() };
        let valid = json_document.parse(tokens).unwrap();
        assert!(valid);
    }
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

    let json_document = &mut JSONDocument { data: Vec::new() };
    let valid = json_document.parse(tokens).unwrap();
    assert!(valid);
}
