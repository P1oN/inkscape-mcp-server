//! Token-aware CSS dependency rewriting. Keep non-URL source text intact.
use cssparser::{Parser, Token};
const REFUSED: &str = "engine input refused: unsupported CSS dependency syntax";

pub fn rewrite(
    value: &str,
    mut replace: impl FnMut(&str) -> Result<Option<String>, String>,
) -> Result<String, String> {
    rewrite_with_imports(value, |url, _| replace(url))
}

pub fn rewrite_with_imports(
    value: &str,
    mut replace: impl FnMut(&str, bool) -> Result<Option<String>, String>,
) -> Result<String, String> {
    let mut parser = Parser::new(value);
    parser.set_nested_block_limit(32);
    let mut urls = Vec::new();
    scan(&mut parser, &mut urls, 0)?;
    let mut result = String::new();
    let mut offset = 0;
    for (start, end, url, import) in urls {
        result.push_str(&value[offset..start]);
        if let Some(replacement) = replace(&url, import)? {
            result.push_str("url(");
            cssparser::serialize_string(&replacement, &mut result).map_err(|_| REFUSED)?;
            result.push(')');
        } else {
            result.push_str(&value[start..end]);
        }
        offset = end;
    }
    result.push_str(&value[offset..]);
    Ok(result)
}

fn scan(
    parser: &mut Parser<'_>,
    urls: &mut Vec<(usize, usize, String, bool)>,
    depth: usize,
) -> Result<(), String> {
    if depth > 32 {
        return Err(REFUSED.into());
    }
    while !parser.is_exhausted() {
        let start = parser.position().byte_index();
        let token = parser
            .next_including_whitespace_and_comments()
            .map_err(|_| REFUSED)?
            .clone();
        match token {
            Token::UnquotedUrl(url) => urls.push((
                start,
                parser.position().byte_index(),
                url.to_string(),
                false,
            )),
            Token::AtKeyword(name) if name.eq_ignore_ascii_case("import") => {
                parser.skip_whitespace();
                let begin = parser.position().byte_index();
                let token = parser.next().map_err(|_| REFUSED)?.clone();
                let url = match token {
                    Token::QuotedString(url) | Token::UnquotedUrl(url) => url.to_string(),
                    Token::Function(name) if name.eq_ignore_ascii_case("url") => parser
                        .parse_nested_block(|inner| {
                            let url = inner.expect_string_cloned()?;
                            inner.expect_exhausted()?;
                            Ok::<_, cssparser::ParseError<()>>(url.to_string())
                        })
                        .map_err(|_| REFUSED)?,
                    _ => return Err(REFUSED.into()),
                };
                urls.push((begin, parser.position().byte_index(), url, true));
            }
            Token::Function(name) if name.eq_ignore_ascii_case("url") => {
                let url: String = parser
                    .parse_nested_block(|inner| {
                        let url = inner.expect_string_cloned()?;
                        inner.expect_exhausted()?;
                        Ok::<_, cssparser::ParseError<()>>(url.to_string())
                    })
                    .map_err(|_| REFUSED)?;
                urls.push((start, parser.position().byte_index(), url, false));
            }
            Token::Function(name)
                if [
                    "image",
                    "image-set",
                    "-webkit-image-set",
                    "src",
                    "expression",
                ]
                .iter()
                .any(|n| name.eq_ignore_ascii_case(n)) =>
            {
                return Err(REFUSED.into());
            }
            Token::Function(_)
            | Token::ParenthesisBlock
            | Token::SquareBracketBlock
            | Token::CurlyBracketBlock => {
                parser
                    .parse_nested_block(|inner| {
                        scan(inner, urls, depth + 1).map_err(|_| cssparser::ParseError {
                            kind: cssparser::ParseErrorKind::Custom(()),
                        })
                    })
                    .map_err(|_| REFUSED)?;
            }
            Token::BadUrl(_)
            | Token::BadString(_)
            | Token::CloseParenthesis
            | Token::CloseSquareBracket
            | Token::CloseCurlyBracket => return Err(REFUSED.into()),
            _ => {}
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn escaped_urls_and_nested_blocks_are_seen_but_strings_are_not_urls() {
        let mut seen = Vec::new();
        let css =
            r#"/*keep*/ .x{fill:u\72l('#g');font-family:'url(/not-a-file)';filter:url("a.svg#f")}"#;
        let result = rewrite(css, |url| {
            seen.push(url.to_owned());
            Ok(None)
        })
        .unwrap();
        assert_eq!(result, css);
        assert_eq!(seen, vec!["#g", "a.svg#f"]);
        let mut imports = Vec::new();
        let css = r#"@\69mport 'a.css' screen;"#;
        assert_eq!(
            rewrite_with_imports(css, |url, import| {
                imports.push((url.to_owned(), import));
                Ok(None)
            })
            .unwrap(),
            css
        );
        assert_eq!(imports, vec![("a.css".into(), true)]);
        assert!(rewrite("x{fill:url(a b)}", |_| Ok(None)).is_err());
    }
}
