use std::collections::HashMap;
use crate::tokenize::Token;
use crate::value::Value;

#[derive(Debug, PartialEq)]
pub enum TokenParseError{
    /// 转义序列在没有4个十六进制数字的情况下启动
    UnfinishedEscape,
    /// 转义序列中的字符不是有效的十六进制字符
    InvalidHexValue,
    /// Unicode 值无效
    InvalidCodePointValue,
    ExpectedComma,
    ExpectedProperty,
    ExpectedColon
}

type ParseResult = Result<Value, TokenParseError>;

pub fn parse_tokens(tokens: &[Token], index: &mut usize) -> ParseResult {
    let token = &tokens[*index];
    if matches!(token,
        Token::Null | Token::False | Token::True | Token::Number(_) | Token::String(_)
    ) {
        *index += 1;
    }
    match token {
        Token::Null => Ok(Value::Null),
        Token::False => Ok(Value::Boolean(false)),
        Token::True => Ok(Value::Boolean(true)),
        Token::Number(number) => Ok(Value::Number(*number)),
        Token::String(string) => parse_string(string),
        Token::LeftBracket => parse_array(tokens, index),
        Token::LeftBrace => parse_object(tokens, index),
        _ => todo!()
    }
}

fn parse_string(input: &str) -> ParseResult {
    let unescaped = unescape_string(input)?;
    Ok(Value::String(unescaped))
}

fn unescape_string(input: &str) -> Result<String, TokenParseError> {
    let mut output = String::new();

    let mut is_escaping = false;
    let mut chars = input.chars();
    while let Some(next_char) = chars.next() {
        if is_escaping {
            match next_char {
                '"' => output.push('"'),
                '\\' => output.push('\\'),
                'b' => output.push('\u{8}'),
                'f' => output.push('\u{12}'),
                'n' => output.push('\n'),
                'r' => output.push('\r'),
                't' => output.push('\t'),
                'u' => {
                    let mut sum = 0;
                    for i in 0..4 {
                        let next_char = chars.next().ok_or(TokenParseError::UnfinishedEscape)?;
                        let digit = next_char.to_digit(16).ok_or(TokenParseError::InvalidHexValue)?;
                        sum += (16u32).pow(3 - i) * digit;
                    }
                    let unescaped_char = char::from_u32(sum).ok_or(TokenParseError::InvalidCodePointValue)?;
                    output.push(unescaped_char);
                },
                _ => output.push(next_char),
            }
            is_escaping = false;
        } else if next_char == '\\' {
            is_escaping = true;
        } else {
            output.push(next_char);
        }
    }
    Ok(output)
}

fn parse_array(tokens: &[Token], index: &mut usize) -> ParseResult {
    let mut array = Vec::new();
    loop {
        *index += 1;
        if tokens[*index] == Token::RightBracket {
            break;
        }
        let value = parse_tokens(tokens, index)?;
        array.push(value);

        let token = &tokens[*index];
        match token {
            Token::Comma => {},
            Token::RightBracket => break,
            _ => return Err(TokenParseError::ExpectedComma)
        }
    }
    *index += 1;
    Ok(Value::Array(array))
}

fn parse_object(tokens: &[Token], index: &mut usize) -> ParseResult {
    let mut map = HashMap::new();

    loop {
        // 消费逗号和左括号
        *index += 1;
        if tokens[*index] == Token::RightBrace {
            break;
        }
        if let Token::String(s) = &tokens[*index] {
            *index += 1;
            if Token::Colon == tokens[*index] {
                *index += 1;
                let key = unescape_string(s)?;
                let value = parse_tokens(tokens, index)?;
                map.insert(key, value);
            } else {
                return Err(TokenParseError::ExpectedColon)
            }
            // 在键值对后面的是 Comma 或 RightBrace
            match &tokens[*index] {
                Token::Comma => {},
                Token::RightBrace => break,
                _ => return Err(TokenParseError::ExpectedComma),
            }
        } else {
            return Err(TokenParseError::ExpectedProperty)
        }
    }
    // 消费右括号
    *index += 1;

    Ok(Value::Object(map))
}
#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use crate::tokenize::Token;
    use crate::value::Value;

    use super::parse_tokens;

    fn check(input: &[Token], expected: Value) {
        let actual = parse_tokens(input, &mut 0).unwrap();
        assert_eq!(actual, expected);
    }

    #[test]
    fn parses_null() {
        check(&[Token::Null], Value::Null);
    }
    #[test]
    fn parses_false() {
        check(&[Token::False], Value::Boolean(false));
    }
    #[test]
    fn parses_true() {
        check(&[Token::True], Value::Boolean(true));
    }
    #[test]
    fn parses_string_no_escapes() {
        let input = [Token::String("hello world".into())];
        let expected = Value::String("hello world".into());
        check(&input, expected);
    }
    #[test]
    fn parses_string_non_ascii() {
        let input = [Token::String("olá_こんにちは_नमस्ते_привіт".into())];
        let expected = Value::String(String::from("olá_こんにちは_नमस्ते_привіт"));

        check(&input, expected);
    }

    #[test]
    fn parses_string_with_emoji() {
        let input = [Token::String("hello 💩 world".into())];
        let expected = Value::String(String::from("hello 💩 world"));

        check(&input, expected);
    }
    #[test]
    fn parses_string_unescape_backslash() {
        let input = [Token::String(r#"hello\\workd"#.into())];
        let expected = Value::String(r#"hello\workd"#.into());
        check(&input, expected);
    }
    #[test]
    fn parses_array_one_element() {
        let input = [Token::LeftBracket, Token::True, Token::RightBracket];
        let expected = Value::Array(vec![Value::Boolean(true)]);
        check(&input, expected);
    }
    #[test]
    fn parses_array_two_elements() {
        let input = [
            Token::LeftBracket,
            Token::Null,
            Token::Comma,
            Token::Number(16.0),
            Token::RightBracket,
        ];
        let expected = Value::Array(vec![Value::Null, Value::Number(16.0)]);
        check(&input, expected);
    }
    #[test]
    fn parses_empty_array() {
        let input = [Token::LeftBracket, Token::RightBracket];
        let expected = Value::Array(vec![]);
        check(&input, expected);
    }
    #[test]
    fn parses_nested_array() {
        let input = [
            Token::LeftBracket,
            Token::Null,
            Token::Comma,
            Token::LeftBracket,
            Token::RightBracket,
            Token::RightBracket,
        ];
        let expected = Value::Array(vec![Value::Null, Value::Array(vec![])]);
        check(&input, expected);
    }
    #[test]
    fn parses_empty_object() {
        let input = [Token::LeftBrace, Token::RightBrace];
        let expected = Value::Object(HashMap::new());

        check(&input, expected);
    }
    #[test]
    fn parses_object_one_element() {
        let input = [
            Token::LeftBrace,
            Token::String("key".into()),
            Token::Colon,
            Token::String("value".into()),
            Token::RightBrace,
        ];
        let mut map: HashMap<String, Value> = HashMap::new();
        map.insert("key".into(), Value::String("value".into()));
        let expected = Value::Object(map);

        check(&input, expected);
    }

    #[test]
    fn parses_object_one_element_comma_end() {
        let input = [
            Token::LeftBrace,
            Token::String("key".into()),
            Token::Colon,
            Token::String("value".into()),
            Token::Comma,
            Token::RightBrace,
        ];
        let mut map: HashMap<String, Value> = HashMap::new();
        map.insert("key".into(), Value::String("value".into()));
        let expected = Value::Object(map);

        check(&input, expected);
    }

    #[test]
    fn parses_object_two_elements() {
        let input = [
            Token::LeftBrace,
            Token::String("key".into()),
            Token::Colon,
            Token::String("value".into()),
            Token::Comma,
            Token::String("key1".into()),
            Token::Colon,
            Token::Number(16.0),
            Token::RightBrace,
        ];
        let mut map: HashMap<String, Value> = HashMap::new();
        map.insert("key".into(), Value::String("value".into()));
        map.insert("key1".into(), Value::Number(16.0));
        let expected = Value::Object(map);

        check(&input, expected);
    }

    #[test]
    fn parses_object_nested_array() {
        let input = [
            Token::LeftBrace,
            Token::String("key".into()),
            Token::Colon,
            Token::String("value".into()),
            Token::Comma,
            Token::String("key1".into()),
            Token::Colon,
            Token::LeftBracket,
            Token::RightBracket,
            Token::RightBrace,
        ];
        let mut map: HashMap<String, Value> = HashMap::new();
        map.insert("key".into(), Value::String("value".into()));
        map.insert("key1".into(), Value::Array(vec![]));
        let expected = Value::Object(map);

        check(&input, expected);
    }

    #[test]
    fn parses_object_nested_object() {
        let input = [
            Token::LeftBrace,
            Token::String("key".into()),
            Token::Colon,
            Token::String("value".into()),
            Token::Comma,
            Token::String("key1".into()),
            Token::Colon,
            Token::LeftBrace,
            Token::RightBrace,
            Token::RightBrace,
        ];
        let mut map: HashMap<String, Value> = HashMap::new();
        map.insert("key".into(), Value::String("value".into()));
        map.insert("key1".into(), Value::Object(HashMap::new()));
        let expected = Value::Object(map);

        check(&input, expected);
    }
}