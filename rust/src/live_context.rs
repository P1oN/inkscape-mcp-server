//! Bounded literal context replies. Accepts data grammar only, never evaluates code.
use crate::{
    live_bus::{Bus, Context, Identity},
    live_socket::Error,
};
use serde_json::{Value, json};
const CAP: usize = 1024 * 1024;
const MAX_ROWS: usize = 10_000;
#[derive(Debug)]
enum Literal {
    String(String),
    Tuple(Vec<Self>),
    List(Vec<Self>),
    Other,
}
struct Parser<'a> {
    text: &'a str,
    pos: usize,
    nodes: usize,
}
impl Parser<'_> {
    fn peek(&self) -> Option<char> {
        self.text[self.pos..].chars().next()
    }
    fn next(&mut self) -> Option<char> {
        let c = self.peek()?;
        self.pos += c.len_utf8();
        Some(c)
    }
    fn ws(&mut self) {
        while self.peek().is_some_and(char::is_whitespace) {
            self.next();
        }
    }
    fn hex(&mut self, count: usize) -> Result<char, ()> {
        let mut number = 0u32;
        for _ in 0..count {
            number = number * 16 + self.next().and_then(|c| c.to_digit(16)).ok_or(())?;
        }
        char::from_u32(number).ok_or(())
    }
    fn quoted(&mut self) -> Result<String, ()> {
        let quote = self.next().ok_or(())?;
        let mut result = String::new();
        loop {
            let c = self.next().ok_or(())?;
            if c == quote {
                return Ok(result);
            }
            if c == '\n' || c == '\r' {
                return Err(());
            }
            if c != '\\' {
                result.push(c);
                continue;
            }
            let escape = self.next().ok_or(())?;
            match escape {
                '\n' => (),
                '\r' => {
                    if self.peek() == Some('\n') {
                        self.next();
                    }
                }
                'n' => result.push('\n'),
                'r' => result.push('\r'),
                't' => result.push('\t'),
                'a' => result.push('\u{7}'),
                'b' => result.push('\u{8}'),
                'f' => result.push('\u{c}'),
                'v' => result.push('\u{b}'),
                '\\' | '\'' | '"' => result.push(escape),
                'x' => result.push(self.hex(2)?),
                'u' => result.push(self.hex(4)?),
                'U' => result.push(self.hex(8)?),
                '0'..='7' => {
                    let mut n = escape.to_digit(8).unwrap();
                    for _ in 0..2 {
                        if let Some(d) = self.peek().and_then(|c| c.to_digit(8)) {
                            self.next();
                            n = n * 8 + d;
                        } else {
                            break;
                        }
                    }
                    result.push(char::from_u32(n).ok_or(())?);
                }
                other => {
                    result.push('\\');
                    result.push(other);
                }
            }
        }
    }
    fn value(&mut self, depth: usize) -> Result<Literal, ()> {
        self.ws();
        self.nodes += 1;
        if depth > 16 || self.nodes > MAX_ROWS * 5 + 32 {
            return Err(());
        }
        match self.peek().ok_or(())? {
            '\'' | '"' => {
                let mut string = self.quoted()?;
                self.ws();
                while matches!(self.peek(), Some('\'' | '"')) {
                    string.push_str(&self.quoted()?);
                    self.ws();
                }
                Ok(Literal::String(string))
            }
            '(' | '[' => {
                let open = self.next().unwrap();
                let close = if open == '(' { ')' } else { ']' };
                let mut values = Vec::new();
                let mut comma = false;
                self.ws();
                if self.peek() == Some(close) {
                    self.next();
                    return Ok(if open == '(' {
                        Literal::Tuple(values)
                    } else {
                        Literal::List(values)
                    });
                }
                loop {
                    values.push(self.value(depth + 1)?);
                    self.ws();
                    if self.peek() == Some(close) {
                        self.next();
                        break;
                    }
                    if self.next() != Some(',') {
                        return Err(());
                    }
                    comma = true;
                    self.ws();
                    if self.peek() == Some(close) {
                        self.next();
                        break;
                    }
                }
                if open == '[' {
                    Ok(Literal::List(values))
                } else if !comma && values.len() == 1 {
                    Ok(values.pop().unwrap())
                } else {
                    Ok(Literal::Tuple(values))
                }
            }
            _ => {
                let start = self.pos;
                while self.peek().is_some_and(|c| {
                    c.is_ascii_alphanumeric() || matches!(c, '+' | '-' | '.' | '_')
                }) {
                    self.next();
                }
                if self.pos == start {
                    return Err(());
                }
                Ok(Literal::Other)
            }
        }
    }
}
fn parse(text: &str) -> Result<Literal, ()> {
    if text.len() > CAP || text.contains('\0') {
        return Err(());
    }
    let mut parser = Parser {
        text,
        pos: 0,
        nodes: 0,
    };
    let value = parser.value(0)?;
    parser.ws();
    if parser.pos != text.len() {
        return Err(());
    }
    Ok(value)
}
fn reference(value: Literal) -> Result<Value, Error> {
    let Literal::Tuple(row) = value else {
        return Err(Error::Context("invalid document context reply"));
    };
    if row.len() != 3 {
        return Err(Error::Context("invalid document context reply"));
    }
    let mut strings = Vec::new();
    for item in row {
        if let Literal::String(s) = item {
            strings.push(s);
        } else {
            return Err(Error::Context("invalid document context reply"));
        }
    }
    Identity::new(&strings[0], &strings[1])
        .map_err(|_| Error::Context("no drawing window available; activate a drawing and retry"))?;
    Ok(
        json!({"window_id":strings[0],"document_id":strings[1],"name":if strings[2].is_empty(){None}else{Some(&strings[2])},"path":null,"object_count":null}),
    )
}
pub fn read_reply(text: &str) -> Result<Value, Error> {
    if text.len() > CAP {
        return Err(Error::Context("document bridge response exceeds size cap"));
    }
    reference(parse(text).map_err(|_| Error::Context("invalid document context reply"))?)
}
pub fn list_reply(text: &str) -> Result<Vec<Value>, Error> {
    if text.len() > CAP {
        return Err(Error::Context("document bridge response exceeds size cap"));
    }
    // Preserve the existing bridge's literal annotation normalization, including titles.
    let text = text.replace("@a(sss) ", "");
    let value = parse(&text).map_err(|_| Error::Context("invalid document list reply"))?;
    let Literal::Tuple(mut outer) = value else {
        return Err(Error::Context("invalid document list reply"));
    };
    if outer.len() != 1 {
        return Err(Error::Context("invalid document list reply"));
    }
    let Literal::List(rows) = outer.pop().unwrap() else {
        return Err(Error::Context("invalid document list reply"));
    };
    if rows.len() > MAX_ROWS {
        return Err(Error::Context("document list exceeds size cap"));
    }
    rows.into_iter().map(reference).collect()
}
pub fn read(bus: &mut Bus) -> Result<Value, Error> {
    read_reply(&bus.context(Context::Get)?)
}
pub fn list(bus: &mut Bus) -> Result<Vec<Value>, Error> {
    list_reply(&bus.context(Context::List)?)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn compiled_reference_context_literals_identity_and_error_precedence_match() {
        let fixture: Value = serde_json::from_str(include_str!(
            "../../migration/contracts/context-reply-cases.json"
        ))
        .unwrap();
        for case in fixture["cases"].as_array().unwrap() {
            let text = case["text"].as_str().unwrap();
            let result = if case["kind"] == "read" {
                read_reply(text)
            } else {
                list_reply(text).map(|rows| json!(rows))
            };
            let actual = match result {
                Ok(value) => json!({"result":value}),
                Err(Error::Context(message)) => json!({"error":message}),
                _ => panic!("wrong parser error"),
            };
            assert_eq!(actual, case["expected"], "{text}");
        }
    }
    #[test]
    fn native_depth_node_utf8_and_response_caps_refuse_before_binding() {
        assert!(read_reply(&"(".repeat(20)).is_err());
        assert!(parse(&format!("([{}],)", "None,".repeat(50_033))).is_err());
        assert_eq!(
            read_reply(&"x".repeat(CAP + 1)),
            Err(Error::Context("document bridge response exceeds size cap"))
        );
        let window = "12345678-1234-1234-1234-123456789abc";
        let doc = "abcdefab-1234-1234-1234-123456789abc";
        assert!(read_reply(&format!("('{window}','{doc}','\0')")).is_err());
        assert!(read_reply(&format!("('{window}','{doc}','\\ud800')")).is_err());
        assert!(read_reply(&format!("('{window}','{doc}','\\UFFFFFFFF')")).is_err());
        let rows = std::iter::repeat_n(format!("('{window}','{doc}','')"), MAX_ROWS + 1)
            .collect::<Vec<_>>()
            .join(",");
        assert_eq!(
            list_reply(&format!("([{rows}],)")),
            Err(Error::Context("document list exceeds size cap"))
        );
    }
}
