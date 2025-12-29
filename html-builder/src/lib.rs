//! # html-builder
//!
//! A minimal, zero-dependency, no-std compatible HTML builder for Rust.
//!
//! ## Example
//!
//! ```rust
//! use html_builder::{Html, Node};
//!
//! let html = Html::new()
//!     .elem("table", |e| e
//!         .attr("class", "table table-sm")
//!         .child("thead", |e| e
//!             .child("tr", |e| e
//!                 .child("th", |e| e.text("Index"))
//!                 .child("th", |e| e.text("Address"))
//!             )
//!         )
//!         .child("tbody", |e| e
//!             .child("tr", |e| e
//!                 .child("td", |e| e.text("0"))
//!                 .child("td", |e| e
//!                     .child("code", |e| e.text("t1abc...xyz"))
//!                 )
//!             )
//!         )
//!     )
//!     .build();
//! ```

#![no_std]

extern crate alloc;

use alloc::string::{String, ToString};
use alloc::vec::Vec;

/// An HTML element with tag, attributes, and children.
#[derive(Debug, Clone)]
pub struct Element {
    tag: String,
    attrs: Vec<(String, String)>,
    children: Vec<Node>,
    self_closing: bool,
}

/// A node in the HTML tree - either an element or text.
#[derive(Debug, Clone)]
pub enum Node {
    Element(Element),
    Text(String),
    Raw(String),
}

/// HTML builder for constructing HTML documents.
#[derive(Debug, Clone, Default)]
pub struct Html {
    nodes: Vec<Node>,
}

impl Element {
    /// Create a new element with the given tag name.
    pub fn new(tag: impl Into<String>) -> Self {
        let tag = tag.into();
        let self_closing = matches!(
            tag.as_str(),
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
                | "source"
                | "track"
                | "wbr"
        );
        Element {
            tag,
            attrs: Vec::new(),
            children: Vec::new(),
            self_closing,
        }
    }

    /// Add an attribute to this element.
    pub fn attr(mut self, name: impl Into<String>, value: impl Into<String>) -> Self {
        self.attrs.push((name.into(), value.into()));
        self
    }

    /// Add a boolean attribute (no value, e.g., `disabled`, `checked`).
    pub fn bool_attr(mut self, name: impl Into<String>) -> Self {
        self.attrs.push((name.into(), String::new()));
        self
    }

    /// Add a class attribute. If class already exists, appends to it.
    pub fn class(mut self, class: impl Into<String>) -> Self {
        let class = class.into();
        if let Some(pos) = self.attrs.iter().position(|(k, _)| k == "class") {
            self.attrs[pos].1.push(' ');
            self.attrs[pos].1.push_str(&class);
        } else {
            self.attrs.push(("class".to_string(), class));
        }
        self
    }

    /// Add an id attribute.
    pub fn id(self, id: impl Into<String>) -> Self {
        self.attr("id", id)
    }

    /// Add a text child node.
    pub fn text(mut self, content: impl Into<String>) -> Self {
        self.children.push(Node::Text(content.into()));
        self
    }

    /// Add raw HTML (not escaped).
    pub fn raw(mut self, html: impl Into<String>) -> Self {
        self.children.push(Node::Raw(html.into()));
        self
    }

    /// Add a child element using a builder function.
    pub fn child<F>(mut self, tag: impl Into<String>, f: F) -> Self
    where
        F: FnOnce(Element) -> Element,
    {
        let child = f(Element::new(tag));
        self.children.push(Node::Element(child));
        self
    }

    /// Add an existing node as a child.
    pub fn node(mut self, node: Node) -> Self {
        self.children.push(node);
        self
    }

    /// Add multiple children from an iterator.
    pub fn children<I, F>(mut self, items: I, f: F) -> Self
    where
        I: IntoIterator,
        F: Fn(I::Item, Element) -> Element,
    {
        for item in items {
            let child = f(item, Element::new(""));
            if !child.tag.is_empty() {
                self.children.push(Node::Element(child));
            }
        }
        self
    }

    /// Conditionally add content.
    pub fn when<F>(self, condition: bool, f: F) -> Self
    where
        F: FnOnce(Self) -> Self,
    {
        if condition { f(self) } else { self }
    }

    /// Conditionally add content with else branch.
    pub fn when_else<F, G>(self, condition: bool, if_true: F, if_false: G) -> Self
    where
        F: FnOnce(Self) -> Self,
        G: FnOnce(Self) -> Self,
    {
        if condition {
            if_true(self)
        } else {
            if_false(self)
        }
    }

    /// Render this element to a string.
    pub fn render(&self) -> String {
        let mut output = String::new();
        self.render_to(&mut output);
        output
    }

    /// Render this element to an existing string buffer.
    pub fn render_to(&self, output: &mut String) {
        output.push('<');
        output.push_str(&self.tag);

        for (name, value) in &self.attrs {
            output.push(' ');
            output.push_str(name);
            if !value.is_empty() {
                output.push_str("=\"");
                output.push_str(&escape_attr(value));
                output.push('"');
            }
        }

        if self.self_closing && self.children.is_empty() {
            output.push_str(" />");
        } else {
            output.push('>');

            for child in &self.children {
                child.render_to(output);
            }

            output.push_str("</");
            output.push_str(&self.tag);
            output.push('>');
        }
    }
}

impl Node {
    /// Render this node to a string.
    pub fn render(&self) -> String {
        let mut output = String::new();
        self.render_to(&mut output);
        output
    }

    /// Render this node to an existing string buffer.
    pub fn render_to(&self, output: &mut String) {
        match self {
            Node::Element(elem) => elem.render_to(output),
            Node::Text(text) => output.push_str(&escape_html(text)),
            Node::Raw(html) => output.push_str(html),
        }
    }
}

impl Html {
    /// Create a new empty HTML builder.
    pub fn new() -> Self {
        Html { nodes: Vec::new() }
    }

    /// Add a root element using a builder function.
    pub fn elem<F>(mut self, tag: impl Into<String>, f: F) -> Self
    where
        F: FnOnce(Element) -> Element,
    {
        let elem = f(Element::new(tag));
        self.nodes.push(Node::Element(elem));
        self
    }

    /// Add a text node at the root level.
    pub fn text(mut self, content: impl Into<String>) -> Self {
        self.nodes.push(Node::Text(content.into()));
        self
    }

    /// Add raw HTML at the root level.
    pub fn raw(mut self, html: impl Into<String>) -> Self {
        self.nodes.push(Node::Raw(html.into()));
        self
    }

    /// Build the final HTML string.
    pub fn build(&self) -> String {
        let mut output = String::new();
        for node in &self.nodes {
            node.render_to(&mut output);
        }
        output
    }
}

/// Escape special HTML characters in text content.
pub fn escape_html(s: &str) -> String {
    let mut output = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '&' => output.push_str("&amp;"),
            '<' => output.push_str("&lt;"),
            '>' => output.push_str("&gt;"),
            _ => output.push(c),
        }
    }
    output
}

/// Escape special characters in attribute values.
pub fn escape_attr(s: &str) -> String {
    let mut output = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '&' => output.push_str("&amp;"),
            '<' => output.push_str("&lt;"),
            '>' => output.push_str("&gt;"),
            '"' => output.push_str("&quot;"),
            '\'' => output.push_str("&#x27;"),
            _ => output.push(c),
        }
    }
    output
}

// Convenience functions for common elements

/// Create a div element.
pub fn div<F>(f: F) -> Element
where
    F: FnOnce(Element) -> Element,
{
    f(Element::new("div"))
}

/// Create a span element.
pub fn span<F>(f: F) -> Element
where
    F: FnOnce(Element) -> Element,
{
    f(Element::new("span"))
}

/// Create a table element.
pub fn table<F>(f: F) -> Element
where
    F: FnOnce(Element) -> Element,
{
    f(Element::new("table"))
}

/// Create a simple text element.
pub fn text_elem(tag: &str, content: impl Into<String>) -> Element {
    Element::new(tag).text(content)
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::vec;

    #[test]
    fn test_simple_element() {
        let html = Element::new("div")
            .class("container")
            .text("Hello")
            .render();
        assert_eq!(html, r#"<div class="container">Hello</div>"#);
    }

    #[test]
    fn test_nested_elements() {
        let html = Html::new()
            .elem("table", |e| {
                e.class("table").child("tr", |e| {
                    e.child("td", |e| e.text("Cell 1"))
                        .child("td", |e| e.text("Cell 2"))
                })
            })
            .build();

        assert_eq!(
            html,
            r#"<table class="table"><tr><td>Cell 1</td><td>Cell 2</td></tr></table>"#
        );
    }

    #[test]
    fn test_attributes() {
        let html = Element::new("input")
            .attr("type", "text")
            .attr("name", "username")
            .bool_attr("disabled")
            .render();

        assert_eq!(html, r#"<input type="text" name="username" disabled />"#);
    }

    #[test]
    fn test_escape_html() {
        let html = Element::new("div")
            .text("<script>alert('xss')</script>")
            .render();
        assert_eq!(
            html,
            r#"<div>&lt;script&gt;alert('xss')&lt;/script&gt;</div>"#
        );
    }

    #[test]
    fn test_escape_attr() {
        let html = Element::new("div")
            .attr("data-value", "say \"hello\"")
            .render();
        assert_eq!(html, r#"<div data-value="say &quot;hello&quot;"></div>"#);
    }

    #[test]
    fn test_class_chaining() {
        let html = Element::new("div")
            .class("btn")
            .class("btn-primary")
            .class("active")
            .render();
        assert_eq!(html, r#"<div class="btn btn-primary active"></div>"#);
    }

    #[test]
    fn test_children_iterator() {
        let items = vec!["Apple", "Banana", "Cherry"];
        let html = Element::new("ul")
            .children(items, |item, _| Element::new("li").text(item))
            .render();

        assert_eq!(
            html,
            r#"<ul><li>Apple</li><li>Banana</li><li>Cherry</li></ul>"#
        );
    }

    #[test]
    fn test_conditional() {
        let show_button = true;
        let html = Element::new("div")
            .when(show_button, |e| e.child("button", |e| e.text("Click me")))
            .render();

        assert_eq!(html, r#"<div><button>Click me</button></div>"#);

        let show_button = false;
        let html = Element::new("div")
            .when(show_button, |e| e.child("button", |e| e.text("Click me")))
            .render();

        assert_eq!(html, r#"<div></div>"#);
    }

    #[test]
    fn test_raw_html() {
        let html = Element::new("div").raw("<strong>Bold</strong>").render();
        assert_eq!(html, r#"<div><strong>Bold</strong></div>"#);
    }

    #[test]
    fn test_self_closing_tags() {
        let html = Element::new("br").render();
        assert_eq!(html, r#"<br />"#);

        let html = Element::new("img").attr("src", "pic.jpg").render();
        assert_eq!(html, r#"<img src="pic.jpg" />"#);
    }

    #[test]
    fn test_address_table_example() {
        let addresses = vec![(0, "t1abc123", "u1xyz789"), (1, "t1def456", "u1uvw012")];

        let html = Html::new()
            .elem("table", |t| {
                t.class("table table-sm")
                    .child("thead", |e| {
                        e.child("tr", |e| {
                            e.child("th", |e| e.text("Index"))
                                .child("th", |e| e.text("Transparent"))
                                .child("th", |e| e.text("Unified"))
                        })
                    })
                    .child("tbody", |e| {
                        e.children(addresses, |(idx, t_addr, u_addr), _| {
                            Element::new("tr")
                                .child("td", |e| e.text(idx.to_string()))
                                .child("td", |e| e.child("code", |e| e.text(t_addr)))
                                .child("td", |e| e.child("code", |e| e.text(u_addr)))
                        })
                    })
            })
            .build();

        assert!(html.contains("<table class=\"table table-sm\">"));
        assert!(html.contains("<code>t1abc123</code>"));
        assert!(html.contains("<code>u1xyz789</code>"));
    }
}
