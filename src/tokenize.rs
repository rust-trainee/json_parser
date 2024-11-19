use std::num::ParseFloatError;
#[derive(Debug, PartialEq)]
pub enum Token {
    /// `{`
    LeftBrace,
    /// `}`
    RightBrace,
    /// `[`
    LeftBracket,
    /// `]`
    RightBracket,
    /// `,`
    Comma,
    /// `:`
    Colon,
    /// `null`
    Null,
    /// `false`
    False,
    /// `true`
    True,
    /// Any number literal
    Number(f64),
    /// Key of the key/value pair or string value
    String(String),
}

#[cfg(test)]
impl Token {
    fn string(input: &str) -> Self {
        Self::String(String::from(input))
    }
}

/// One of the possible errors that could occur while tokenizing the input
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum TokenizeError {
    /// The input appeared to be the start of a literal value but did not finish
    UnfinishedLiteralValue,
    /// Unable to parse the float
    ParseNumberError(ParseFloatError),
    /// String was never completed
    UnclosedQuotes,
    /// Character is not part of a JSON token
    CharNotRecognized(char)
}

pub fn tokenize(input: &str) -> Result<Vec<Token>, TokenizeError>{
    let chars: Vec<_> = input.chars().collect();
    let mut index = 0;

    let mut tokens = Vec::new();
    while index < chars.len() {
        if !chars[index].is_whitespace() {
            let token = make_token(&chars, &mut index)?;
            tokens.push(token);
        }
        index += 1;
    }

    Ok(tokens)
}

fn make_token(chars: &[char], index: &mut usize) -> Result<Token, TokenizeError> {
    let ch = chars[*index];
    let token = match ch {
        '[' => Token::LeftBracket,
        ']' => Token::RightBracket,
        '{' => Token::LeftBrace,
        '}' => Token::RightBrace,
        ',' => Token::Comma,
        ':' => Token::Colon,
        'n' => tokenize_literal(chars, index, "null", Token::Null)?,
        't' => tokenize_literal(chars, index, "true", Token::True)?,
        'f' => tokenize_literal(chars, index, "false", Token::False)?,
        c if c.is_ascii_digit() => tokenize_float(chars, index)?,
        '"' => tokenize_string(chars, index)?,
        ch => return Err(TokenizeError::CharNotRecognized(ch))
    };

    Ok(token)
}

fn tokenize_literal(chars: &[char], index: &mut usize, literal: &str, token: Token) -> Result<Token, TokenizeError> {
    for expected_char in literal.chars() {
        if expected_char != chars[*index] {
            return Err(TokenizeError::UnfinishedLiteralValue);
        }
        *index += 1;
    }
    *index -= 1;
    Ok(token)
}

fn tokenize_float(chars: &[char], index: &mut usize) -> Result<Token, TokenizeError> {
    let mut unparsed_num = String::new();
    let mut has_decimal = false;

    while *index < chars.len() {
        let ch = chars[*index];
        match ch {
            c if c.is_ascii_digit() => unparsed_num.push(c),
            c if c == '.' && !has_decimal => {
                unparsed_num.push(c);
                has_decimal = true;
            }
            _ => break,
        }
        *index += 1;
    }
    // 回退一个字符
    *index -= 1;
    let num = unparsed_num.parse()
        .map(|f| Token::Number(f))
        .map_err(|err| TokenizeError::ParseNumberError(err))?;
    Ok(num)
}

fn tokenize_string(chars: &[char], index: &mut usize) -> Result<Token, TokenizeError> {
    let mut string = String::new();
    let mut is_escaping = false;

    loop {
        *index += 1;
        if *index >= chars.len() {
            return Err(TokenizeError::UnclosedQuotes);
        }
        let ch = chars[*index];

        match ch {
            '"' if !is_escaping => break,
            '\\' => is_escaping = !is_escaping,
            _ => is_escaping = false,
        }
        string.push(ch);
    }
    Ok(Token::String(string))
}

#[cfg(test)]
mod tests {
    use super::{tokenize, Token, TokenizeError};

    #[test]
    fn just_comma() {
        let input = String::from(",");
        let expected = [Token::Comma];

        let actual = tokenize(&input).unwrap();
        assert_eq!(actual, expected);
    }

    #[test]
    fn all_punctuation() {
        let input = String::from("[{]},:");
        let expected = [
            Token::LeftBracket,
            Token::LeftBrace,
            Token::RightBracket,
            Token::RightBrace,
            Token::Comma,
            Token::Colon,
        ];
        let actual = tokenize(&input).unwrap();
        assert_eq!(actual, expected);
    }

    #[test]
    fn just_null() {
        let input = String::from("null");
        let expected = [Token::Null];
        let actual = tokenize(&input).unwrap();
        assert_eq!(actual, expected);
    }
    #[test]
    fn just_false() {
        let input = String::from("false");
        let expected = [Token::False];
        let actual = tokenize(&input).unwrap();
        assert_eq!(actual, expected);
    }
    #[test]
    fn just_true() {
        let input = String::from("true");
        let expected = [Token::True];
        let actual = tokenize(&input).unwrap();
        assert_eq!(actual, expected);
    }
    #[test]
    fn true_comma() {
        let input = String::from("true,");
        let expected = [Token::True, Token::Comma];

        let actual = tokenize(&input).unwrap();
        assert_eq!(actual, expected);
    }
    #[test]
    fn integer() {
        let input = String::from("123");
        let expected = [Token::Number(123.0)];

        let actual = tokenize(&input).unwrap();

        assert_eq!(actual, expected);
    }
    #[test]
    fn floating_point() {
        let input = String::from("1.23");
        let expected = [Token::Number(1.23)];

        let actual = tokenize(&input).unwrap();
        assert_eq!(actual, expected);
    }

    #[test]
    fn just_ken() {
        let input = String::from("\"ken\"");
        let expected = [Token::String(String::from("ken"))];

        let actual = tokenize(&input).unwrap();
        assert_eq!(actual, expected);
    }

    #[test]
    fn unclosed_string() {
        let input = String::from("\"unclosed");
        let expected = Err(TokenizeError::UnclosedQuotes);

        let actual = tokenize(&input);
        assert_eq!(actual, expected);
    }

    #[test]
    fn escaped_quote() {
        let input = String::from(r#""the \" is OK""#);
        let expected = [Token::string(r#"the \" is OK"#)];

        let actual = tokenize(&input).unwrap();

        assert_eq!(actual, expected);
    }
}