//! Pure-Rust declarative HTML-fragment compiler producing safe DOM node trees
//! for Trusted Types / strict CSP pages (mounted via createElement/createElementNS/createTextNode).
use injectra_core::{Error, ErrorKind, Result};
use serde_json::{Value, json};

const MAX_DEPTH: usize = 64;
const MAX_NODES: usize = 4096;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DomNode {
    Text(String),
    Element {
        tag: String,
        svg: bool,
        attrs: Vec<(String, String)>,
        children: Vec<DomNode>,
    },
}

impl DomNode {
    pub fn to_json(&self) -> Value {
        match self {
            Self::Text(text) => Value::String(text.clone()),
            Self::Element {
                tag,
                svg,
                attrs,
                children,
            } => {
                let mut obj = serde_json::Map::new();
                obj.insert("t".into(), Value::String(tag.clone()));
                if *svg {
                    obj.insert("ns".into(), Value::String("svg".into()));
                }
                if !attrs.is_empty() {
                    obj.insert(
                        "a".into(),
                        Value::Array(attrs.iter().map(|(k, v)| json!([k, v])).collect()),
                    );
                }
                if !children.is_empty() {
                    obj.insert(
                        "c".into(),
                        Value::Array(children.iter().map(Self::to_json).collect()),
                    );
                }
                Value::Object(obj)
            }
        }
    }
}

fn is_void_html_tag(lower: &str) -> bool {
    matches!(
        lower,
        "area"
            | "base"
            | "br"
            | "col"
            | "embed"
            | "hr"
            | "img"
            | "input"
            | "link"
            | "meta"
            | "param"
            | "source"
            | "track"
            | "wbr"
    )
}

fn decode_entities(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let bytes = input.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'&'
            && let Some(semi_rel) = input[i + 1..].find(';')
            && semi_rel > 0
            && semi_rel <= 12
        {
            let entity = &input[i + 1..i + 1 + semi_rel];
            let decoded = match entity {
                "amp" => Some('&'),
                "lt" => Some('<'),
                "gt" => Some('>'),
                "quot" => Some('"'),
                "apos" => Some('\''),
                "nbsp" => Some('\u{00a0}'),
                _ if entity.starts_with("#x") || entity.starts_with("#X") => {
                    u32::from_str_radix(&entity[2..], 16)
                        .ok()
                        .and_then(char::from_u32)
                }
                _ if entity.starts_with('#') => {
                    entity[1..].parse::<u32>().ok().and_then(char::from_u32)
                }
                _ => None,
            };
            if let Some(ch) = decoded {
                out.push(ch);
                i += semi_rel + 2;
                continue;
            }
        }
        let ch = input[i..].chars().next().unwrap();
        out.push(ch);
        i += ch.len_utf8();
    }
    out
}

fn is_name_char(b: u8) -> bool {
    b.is_ascii_alphanumeric() || matches!(b, b'-' | b'_' | b':' | b'.')
}

struct Parser<'a> {
    src: &'a str,
    bytes: &'a [u8],
    pos: usize,
    node_count: usize,
}

impl<'a> Parser<'a> {
    fn new(src: &'a str) -> Self {
        Self {
            src,
            bytes: src.as_bytes(),
            pos: 0,
            node_count: 0,
        }
    }

    fn err(&self, msg: impl Into<String>) -> Error {
        Error::prepare(ErrorKind::InvalidInput, msg)
    }

    fn skip_ws(&mut self) {
        while self.pos < self.bytes.len() && self.bytes[self.pos].is_ascii_whitespace() {
            self.pos += 1;
        }
    }

    fn bump_node(&mut self) -> Result<()> {
        self.node_count += 1;
        if self.node_count > MAX_NODES {
            return Err(self.err("declarative DOM exceeds maximum node limit"));
        }
        Ok(())
    }

    fn parse_nodes(
        &mut self,
        parent_tag: Option<&str>,
        in_svg: bool,
        depth: usize,
    ) -> Result<Vec<DomNode>> {
        if depth > MAX_DEPTH {
            return Err(self.err("declarative DOM exceeds maximum nesting depth"));
        }
        let mut nodes = Vec::new();
        while self.pos < self.bytes.len() {
            if self.bytes[self.pos] == b'<' {
                if self.src[self.pos..].starts_with("<!--") {
                    let Some(end) = self.src[self.pos + 4..].find("-->") else {
                        return Err(self.err("unterminated HTML comment in declarative DOM"));
                    };
                    self.pos += 4 + end + 3;
                    continue;
                }
                if self.src[self.pos..].starts_with("</") {
                    let close_start = self.pos;
                    self.pos += 2;
                    self.skip_ws();
                    let name_start = self.pos;
                    while self.pos < self.bytes.len() && is_name_char(self.bytes[self.pos]) {
                        self.pos += 1;
                    }
                    let close_tag = &self.src[name_start..self.pos];
                    self.skip_ws();
                    if self.pos >= self.bytes.len() || self.bytes[self.pos] != b'>' {
                        return Err(self.err("malformed closing tag in declarative DOM"));
                    }
                    self.pos += 1;
                    if let Some(expected) = parent_tag
                        && close_tag.eq_ignore_ascii_case(expected)
                    {
                        return Ok(nodes);
                    }
                    return Err(self.err(format!(
                        "unexpected closing tag </{close_tag}> at byte {close_start}"
                    )));
                }
                if self.src[self.pos..].starts_with("<!") {
                    return Err(self.err("DOCTYPE or processing declarations are not supported in declarative DOM fragments"));
                }
                let elem = self.parse_element(in_svg, depth)?;
                nodes.push(elem);
            } else {
                let start = self.pos;
                while self.pos < self.bytes.len() && self.bytes[self.pos] != b'<' {
                    self.pos += 1;
                }
                let text = decode_entities(&self.src[start..self.pos]);
                if !text.is_empty() {
                    self.bump_node()?;
                    nodes.push(DomNode::Text(text));
                }
            }
        }
        if let Some(expected) = parent_tag {
            return Err(self.err(format!("unclosed tag <{expected}> in declarative DOM")));
        }
        Ok(nodes)
    }

    fn parse_element(&mut self, in_svg: bool, depth: usize) -> Result<DomNode> {
        self.pos += 1; // '<'
        let tag_start = self.pos;
        if self.pos >= self.bytes.len() || !self.bytes[self.pos].is_ascii_alphabetic() {
            return Err(self.err("invalid HTML tag name in declarative DOM"));
        }
        while self.pos < self.bytes.len() && is_name_char(self.bytes[self.pos]) {
            self.pos += 1;
        }
        let raw_tag = &self.src[tag_start..self.pos];
        let lower_tag = raw_tag.to_ascii_lowercase();
        if lower_tag == "script" {
            return Err(self.err(
                "declarative DOM mode rejects <script> elements; put behavior in the bundle JS file",
            ));
        }
        let svg = in_svg || lower_tag == "svg";
        let tag = if svg {
            raw_tag.to_owned()
        } else {
            lower_tag.clone()
        };

        let mut attrs = Vec::new();
        let mut self_closing = false;
        loop {
            self.skip_ws();
            if self.pos >= self.bytes.len() {
                return Err(self.err(format!("unterminated opening tag <{raw_tag}>")));
            }
            if self.bytes[self.pos] == b'>' {
                self.pos += 1;
                break;
            }
            if self.src[self.pos..].starts_with("/>") {
                self.pos += 2;
                self_closing = true;
                break;
            }
            let attr_start = self.pos;
            while self.pos < self.bytes.len() && is_name_char(self.bytes[self.pos]) {
                self.pos += 1;
            }
            if self.pos == attr_start {
                return Err(self.err(format!("invalid attribute syntax inside <{raw_tag}>")));
            }
            let raw_attr = &self.src[attr_start..self.pos];
            let lower_attr = raw_attr.to_ascii_lowercase();
            if lower_attr.starts_with("on") && lower_attr.len() > 2 {
                return Err(self.err(format!(
                    "declarative DOM mode rejects inline event handler attribute {raw_attr}; bind listeners in JS"
                )));
            }
            let attr_name = if svg { raw_attr.to_owned() } else { lower_attr };
            self.skip_ws();
            let attr_val = if self.pos < self.bytes.len() && self.bytes[self.pos] == b'=' {
                self.pos += 1;
                self.skip_ws();
                if self.pos >= self.bytes.len() {
                    return Err(self.err(format!("missing value for attribute {attr_name}")));
                }
                let quote = self.bytes[self.pos];
                if quote == b'"' || quote == b'\'' {
                    self.pos += 1;
                    let val_start = self.pos;
                    while self.pos < self.bytes.len() && self.bytes[self.pos] != quote {
                        self.pos += 1;
                    }
                    if self.pos >= self.bytes.len() {
                        return Err(
                            self.err(format!("unterminated attribute value for {attr_name}"))
                        );
                    }
                    let val = decode_entities(&self.src[val_start..self.pos]);
                    self.pos += 1;
                    val
                } else {
                    let val_start = self.pos;
                    while self.pos < self.bytes.len()
                        && !self.bytes[self.pos].is_ascii_whitespace()
                        && self.bytes[self.pos] != b'>'
                    {
                        self.pos += 1;
                    }
                    decode_entities(&self.src[val_start..self.pos])
                }
            } else {
                String::new()
            };
            attrs.push((attr_name, attr_val));
        }

        self.bump_node()?;
        let children = if self_closing || (!svg && is_void_html_tag(&lower_tag)) {
            Vec::new()
        } else {
            let child_svg = svg && lower_tag != "foreignobject";
            self.parse_nodes(Some(raw_tag), child_svg, depth + 1)?
        };
        Ok(DomNode::Element {
            tag,
            svg,
            attrs,
            children,
        })
    }
}

pub fn compile_html_to_dom(html: &str) -> Result<Vec<Value>> {
    let mut parser = Parser::new(html);
    let nodes = parser.parse_nodes(None, false, 0)?;
    Ok(nodes.iter().map(DomNode::to_json).collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compiles_controls_entities_void_tags_and_svg_namespace() {
        let dom = compile_html_to_dom(
            r#"<!-- comment --><button type="button" data-v="a&amp;b" disabled>&lt;Go&gt;</button><input value="x"><svg viewBox="0 0 10 10"><circle r="4"/></svg>"#,
        )
        .unwrap();
        assert_eq!(dom.len(), 3);
        assert_eq!(dom[0]["t"], "button");
        assert_eq!(
            dom[0]["a"],
            json!([["type", "button"], ["data-v", "a&b"], ["disabled", ""]])
        );
        assert_eq!(dom[0]["c"], json!(["<Go>"]));
        assert_eq!(dom[1]["t"], "input");
        assert_eq!(dom[2]["t"], "svg");
        assert_eq!(dom[2]["ns"], "svg");
        assert_eq!(dom[2]["a"], json!([["viewBox", "0 0 10 10"]]));
        assert_eq!(dom[2]["c"][0]["t"], "circle");
        assert_eq!(dom[2]["c"][0]["ns"], "svg");
    }

    #[test]
    fn rejects_script_tags_inline_handlers_and_malformed_markup() {
        assert_eq!(
            compile_html_to_dom("<script>alert(1)</script>")
                .unwrap_err()
                .kind,
            ErrorKind::InvalidInput
        );
        assert_eq!(
            compile_html_to_dom(r#"<button onclick="x()">Hi</button>"#)
                .unwrap_err()
                .kind,
            ErrorKind::InvalidInput
        );
        assert_eq!(
            compile_html_to_dom("<div><span></div>").unwrap_err().kind,
            ErrorKind::InvalidInput
        );
    }
}
