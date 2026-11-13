//! Filter expression tokenizer (internal to `filter`).

#[derive(Debug, Clone, PartialEq)]
pub(super) enum Tok {
    Ident(String),
    Str(String),
    Num(f64),
    Bool(bool),
    Null,
    Op(&'static str),
    LParen,
    RParen,
    And,
    Or,
    Not,
    Eof,
}

pub(super) fn tokenize(input: &str) -> Result<Vec<Tok>, String> {
    let b = input.as_bytes();
    let mut i = 0;
    let mut out = Vec::new();
    while i < b.len() {
        let c = b[i] as char;
        match c {
            c if c.is_whitespace() => i += 1,
            '(' => {
                out.push(Tok::LParen);
                i += 1;
            }
            ')' => {
                out.push(Tok::RParen);
                i += 1;
            }
            '\'' | '"' => {
                let quote = c;
                i += 1;
                let mut s = String::new();
                while i < b.len() && b[i] as char != quote {
                    if b[i] == b'\\' && i + 1 < b.len() {
                        i += 1;
                        s.push(match b[i] as char {
                            'n' => '\n',
                            't' => '\t',
                            other => other,
                        });
                    } else {
                        s.push(b[i] as char);
                    }
                    i += 1;
                }
                if i >= b.len() {
                    return Err("unterminated string".into());
                }
                i += 1;
                out.push(Tok::Str(s));
            }
            '&' if b.get(i + 1) == Some(&b'&') => {
                out.push(Tok::And);
                i += 2;
            }
            '|' if b.get(i + 1) == Some(&b'|') => {
                out.push(Tok::Or);
                i += 2;
            }
            '!' if b.get(i + 1) == Some(&b'=') => {
                out.push(Tok::Op("!="));
                i += 2;
            }
            '!' if b.get(i + 1) == Some(&b'~') => {
                out.push(Tok::Op("!~"));
                i += 2;
            }
            '!' => {
                out.push(Tok::Not);
                i += 1;
            }
            '=' | '>' | '<' | '~' => {
                let two = if i + 1 < b.len() {
                    &input[i..(i + 2).min(input.len())]
                } else {
                    ""
                };
                if two == ">=" || two == "<=" {
                    out.push(Tok::Op(if two == ">=" { ">=" } else { "<=" }));
                    i += 2;
                } else {
                    out.push(Tok::Op(match c {
                        '=' => "=",
                        '>' => ">",
                        '<' => "<",
                        _ => "~",
                    }));
                    i += 1;
                }
            }
            c if c.is_ascii_digit()
                || ((c == '-' || c == '.') && b.get(i + 1).is_some_and(|n| n.is_ascii_digit())) =>
            {
                let start = i;
                i += 1;
                while i < b.len() && (b[i].is_ascii_digit() || b[i] == b'.') {
                    i += 1;
                }
                let text = &input[start..i];
                match text.parse::<f64>() {
                    Ok(n) => out.push(Tok::Num(n)),
                    Err(_) => return Err(format!("bad number '{text}'")),
                }
            }
            c if c.is_alphabetic() || c == '_' || c == '@' => {
                let start = i;
                i += 1;
                while i < b.len() {
                    let ch = b[i] as char;
                    if ch.is_alphanumeric() || ch == '_' || ch == '.' || ch == '@' {
                        i += 1;
                    } else {
                        break;
                    }
                }
                match &input[start..i] {
                    "true" => out.push(Tok::Bool(true)),
                    "false" => out.push(Tok::Bool(false)),
                    "null" => out.push(Tok::Null),
                    word => out.push(Tok::Ident(word.to_string())),
                }
            }
            other => return Err(format!("unexpected character '{other}'")),
        }
    }
    out.push(Tok::Eof);
    Ok(out)
}
