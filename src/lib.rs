use std::{env, fs, path::Path};

#[derive(Debug, PartialEq)]
enum Identfiers {
    LeftBrace,
    RightBrace,
}

fn tokenize(input: String) -> Result<Vec<Identfiers>, ParserErrors> {
    let mut tokens: Vec<Identfiers> = Vec::new();
    let mut chars = input.chars();
    while let Some(c) = chars.next() {
        match c {
            '{' => tokens.push(Identfiers::LeftBrace),
            '}' => tokens.push(Identfiers::RightBrace),
            _ => return Err(ParserErrors::TokenizeError),
        }
    }
    Ok(tokens)
}

fn parse(tokens: Vec<Identfiers>) -> bool {
    if tokens.len() < 2 {
        return false;
    }
    if tokens[0] != Identfiers::LeftBrace || tokens[1] != Identfiers::RightBrace {
        return false;
    }
    true
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
    #[error("invalid json")]
    ParsingError,
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
    let valid_json = parse(tokens);
    if !valid_json {
        return Err(ParserErrors::ParsingError);
    }
    println!("valid json!");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tokenize_on_braces() {
        let tokens = tokenize("{}".into()).unwrap();

        assert_eq!(Identfiers::LeftBrace, tokens[0]);

        assert_eq!(Identfiers::RightBrace, tokens[1]);
    }

    //i think it shouldn't error on unkown chars that is the job of the
    // parser
    #[test]
    fn test_tokenize_errors_on_unkown() {}

    #[test]
    fn test_parse_works_on_valid_tokens() {
        let tokens: Vec<Identfiers> = vec![Identfiers::LeftBrace, Identfiers::RightBrace];

        assert!(parse(tokens));
    }

    #[test]
    fn test_parse_fails_on_invalid_tokens() {
        let tokens: Vec<Identfiers> = vec![Identfiers::RightBrace, Identfiers::RightBrace];

        assert!(!parse(tokens));
    }

    #[test]
    fn test_parse_fails_on_short_tokens() {
        let tokens: Vec<Identfiers> = Vec::new();
        assert!(!parse(tokens));
    }
}
