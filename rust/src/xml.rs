//! Safe DOM candidate: libxml2 retains namespaces, mixed content and entity nodes.
//! Parse flags mirror Python's hardened lxml configuration; never use crate defaults.
//! Ordinary SVG attributes must use get_property_no_ns/remove_property_no_ns:
//! libxml's get_property/remove_property match local names across namespaces.
//! Use explicit *_ns methods for XLink, Inkscape and other qualified metadata.

use libxml::tree::Document;

pub fn parse(bytes: &[u8], max_bytes: usize) -> Result<Document, &'static str> {
    if bytes.len() > max_bytes {
        return Err("input file exceeds the configured size limit");
    }
    let length =
        i32::try_from(bytes.len()).map_err(|_| "input file exceeds the configured size limit")?;
    libxml::init_parser();
    // Own the context so namespace errors cannot be mistaken for successful
    // parsing merely because libxml2 returned a non-null document pointer.
    struct Context(libxml::bindings::xmlParserCtxtPtr);
    impl Drop for Context {
        fn drop(&mut self) {
            // SAFETY: this uniquely owned context was allocated by libxml2.
            unsafe { libxml::bindings::xmlFreeParserCtxt(self.0) };
        }
    }
    // SAFETY: libxml2 has been initialized; no borrowed parser state is shared.
    let context = Context(unsafe { libxml::bindings::xmlNewParserCtxt() });
    if context.0.is_null() {
        return Err("document could not be parsed safely");
    }
    // NONET | NOERROR | NOWARNING | NOCDATA. The crate's ParserOptions omits
    // NOCDATA, which lxml enables by default. Do not enable NOENT, DTDLOAD,
    // RECOVER or HUGE. The buffer remains alive throughout this synchronous call.
    let pointer = unsafe {
        libxml::bindings::xmlCtxtReadMemory(
            context.0,
            bytes.as_ptr().cast(),
            length,
            std::ptr::null(),
            std::ptr::null(),
            2048 | 32 | 64 | 16384,
        )
    };
    if pointer.is_null() {
        Err("document could not be parsed safely")
    } else {
        // Ownership is transferred exactly once to Document's destructor.
        let document = Document::new_ptr(pointer);
        // SAFETY: the live parser context has completed its synchronous read.
        if unsafe { (*context.0).wellFormed == 0 || (*context.0).nsWellFormed == 0 } {
            Err("document could not be parsed safely")
        } else {
            Ok(document)
        }
    }
}

/// Match the reference working-copy serializer's UTF-8 declaration and final newline.
pub fn serialize(document: &Document) -> Vec<u8> {
    if let Some(root) = document.get_root_element()
        && (root.get_prev_sibling().is_some() || root.get_next_sibling().is_some())
    {
        let mut first = root;
        while let Some(previous) = first.get_prev_sibling() {
            first = previous;
        }
        let mut text = String::from("<?xml version='1.0' encoding='UTF-8'?>\n");
        let mut next = Some(first);
        while let Some(node) = next {
            next = node.get_next_sibling();
            text.push_str(&document.node_to_string(&node));
            if node.get_type() == Some(libxml::tree::NodeType::DTDNode) {
                text.push('\n');
            }
        }
        return text.into_bytes();
    }
    let mut text = document.to_string_with_options(libxml::tree::document::SaveOptions::default());
    if text.starts_with("<?xml version=\"1.0\" encoding=\"UTF-8\"?>") {
        text = text.replacen(
            "<?xml version=\"1.0\" encoding=\"UTF-8\"?>",
            "<?xml version='1.0' encoding='UTF-8'?>",
            1,
        );
    }
    if text.ends_with('\n') {
        text.pop();
    }
    text.into_bytes()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn top_level_comments_pis_and_doctype_match_reference_spacing() {
        for (source, body) in [
            (
                "<!--before--><svg/><!--after-->",
                "<!--before--><svg/><!--after-->",
            ),
            (
                "<?foo before?><svg/><?foo after?>",
                "<?foo before?><svg/><?foo after?>",
            ),
            ("<!DOCTYPE svg><svg/>", "<!DOCTYPE svg>\n<svg/>"),
            (
                "<!--before--><!DOCTYPE svg><svg/><!--after-->",
                "<!--before--><!DOCTYPE svg>\n<svg/><!--after-->",
            ),
        ] {
            let document = parse(source.as_bytes(), 4096).unwrap();
            assert_eq!(
                serialize(&document),
                format!("<?xml version='1.0' encoding='UTF-8'?>\n{body}").into_bytes()
            );
        }
    }

    #[test]
    fn preserves_mixed_content_namespace_and_reference() {
        let source = br##"<svg xmlns="http://www.w3.org/2000/svg" xmlns:xlink="http://www.w3.org/1999/xlink"><text id="t">before<tspan>middle</tspan>after</text><use xlink:href="#t"/></svg>"##;
        let document = parse(source, 4096).unwrap();
        let serialized = document.to_string();
        assert!(serialized.contains("before<tspan>middle</tspan>after"));
        assert!(serialized.contains("xlink:href=\"#t\""));
        assert_eq!(
            document
                .get_root_element()
                .unwrap()
                .get_namespace()
                .unwrap()
                .get_href(),
            "http://www.w3.org/2000/svg"
        );
    }

    #[test]
    fn refuses_malformed_and_size_limit() {
        assert!(parse(b"<svg><g></svg>", 4096).is_err());
        assert!(parse(b"<svg/>", 3).is_err());
        assert!(parse(br##"<svg><use x:href="#r"/></svg>"##, 4096).is_err());
        assert!(parse(b"<x:svg/>", 4096).is_err());
        assert!(parse(br#"<svg xmlns:x="relative"><x:g/></svg>"#, 4096).is_ok());
    }

    #[test]
    fn external_entity_is_not_expanded() {
        let document = parse(
            br#"<!DOCTYPE svg [<!ENTITY e SYSTEM "file:///etc/passwd">]><svg>&e;</svg>"#,
            4096,
        )
        .unwrap();
        assert!(
            !document
                .get_root_element()
                .unwrap()
                .get_content()
                .contains("root:")
        );
        assert!(document.to_string().contains("&e;"));
    }

    #[test]
    fn respects_declared_encoding() {
        let document = parse(
            b"<?xml version=\"1.0\" encoding=\"ISO-8859-1\"?><svg><text>caf\xe9</text></svg>",
            4096,
        )
        .unwrap();
        assert_eq!(
            document.get_root_element().unwrap().get_content(),
            "caf\u{e9}"
        );
    }

    #[test]
    fn strips_cdata_without_expanding_internal_entities() {
        let document = parse(
            br#"<!DOCTYPE svg [<!ENTITY e "test">]><svg><text>a&e;b<![CDATA[c<d]]></text></svg>"#,
            4096,
        )
        .unwrap();
        let text = String::from_utf8(serialize(&document)).unwrap();
        assert!(text.contains("<text>a&e;bc&lt;d</text>"));
        assert!(!text.contains("<![CDATA["));
    }

    #[test]
    fn refuses_excessive_depth_and_entity_amplification() {
        let deep = format!("<svg>{}{}</svg>", "<g>".repeat(300), "</g>".repeat(300));
        assert!(parse(deep.as_bytes(), 4096).is_err());
        let mut source = String::from("<!DOCTYPE svg [<!ENTITY e0 'tenletters'>");
        for level in 1..10 {
            source.push_str(&format!(
                "<!ENTITY e{level} '{}'>",
                format!("&e{};", level - 1).repeat(10)
            ));
        }
        source.push_str("]><svg>&e9;</svg>");
        assert!(parse(source.as_bytes(), 4096).is_err());
    }
}
