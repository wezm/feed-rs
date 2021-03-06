use std::io::Read;

use xml::reader as xml_reader;

use crate::model;
use crate::util::attr_value;
use crate::util::element_source::ElementSource;

mod atom;
mod rss0;
mod rss1;
mod rss2;

pub type ParseFeedResult<T> = std::result::Result<T, ParseFeedError>;

/// An error returned when parsing a feed from a source fails
#[derive(Debug)]
pub enum ParseFeedError {
    // TODO add line number/position
    ParseError(ParseErrorKind),
    // Underlying issue with XML (poorly formatted etc)
    XmlReader(xml_reader::Error),
}

impl From<xml_reader::Error> for ParseFeedError {
    fn from(err: xml_reader::Error) -> Self {
        ParseFeedError::XmlReader(err)
    }
}

/// Underlying cause of the parse failure
#[derive(Debug)]
pub enum ParseErrorKind {
    /// Could not find the expected root element (e.g. "channel" for RSS 2)
    NoFeedRoot,
    /// The content type is unsupported and we cannot parse the value into a known representation
    UnknownMimeType(String),
    /// Required content within the source was not found e.g. the XML child text element for a "content" element
    MissingContent(&'static str),
    /// The date/time string was not valid
    InvalidDateTime(Box<dyn std::error::Error>),
}

/// Parse the XML input (Atom or a flavour of RSS) into our model
///
/// # Arguments
///
/// * `input` - A source of XML content such as a string, file etc.
///
/// # Examples
///
/// ```
/// use feed_rs::parser;
/// let xml = r#"
/// <feed>
///    <title type="text">sample feed</title>
///    <updated>2005-07-31T12:29:29Z</updated>
///    <id>feed1</id>
///    <entry>
///        <title>sample entry</title>
///        <id>entry1</id>
///    </entry>
/// </feed>
/// "#;
/// let feed = parser::parse(xml.as_bytes()).unwrap();
/// ```
pub fn parse<R: Read>(input: R) -> ParseFeedResult<model::Feed> {
    // Set up the source of XML elements from the input
    let source = ElementSource::new(input);

    if let Ok(Some(root)) = source.root() {
        // Dispatch to the correct parser
        let version = attr_value(&root.attributes, "version");
        match (root.name.local_name.as_str(), version) {
            ("feed", _) => return atom::parse(root),
            ("rss", Some("2.0")) => return rss2::parse(root),
            ("rss", Some("0.91")) | ("rss", Some("0.92")) => return rss0::parse(root),
            ("RDF", _) => return rss1::parse(root),
            _ => {}
        };
    }

    // Couldn't find a recognised feed within the provided XML stream
    Err(ParseFeedError::ParseError(ParseErrorKind::NoFeedRoot))
}
