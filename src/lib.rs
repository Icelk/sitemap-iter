use log::{error, warn};
use std::fmt::Debug;
use std::str::FromStr;

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum FrequencyParseError {
    InvalidFrequency,
}
/// The frequency of change to a page.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Frequency {
    Always,
    Hourly,
    Daily,
    Weekly,
    Monthly,
    Yearly,
    Never,
}
impl FromStr for Frequency {
    type Err = FrequencyParseError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(if s.eq_ignore_ascii_case("always") {
            Self::Always
        } else if s.eq_ignore_ascii_case("hourly") {
            Self::Hourly
        } else if s.eq_ignore_ascii_case("daily") {
            Self::Daily
        } else if s.eq_ignore_ascii_case("weekly") {
            Self::Weekly
        } else if s.eq_ignore_ascii_case("monthly") {
            Self::Monthly
        } else if s.eq_ignore_ascii_case("yearly") {
            Self::Yearly
        } else if s.eq_ignore_ascii_case("never") {
            Self::Never
        } else {
            return Err(FrequencyParseError::InvalidFrequency);
        })
    }
}
/// The data of a entry in the `urlset`.
///
/// See the [official spec](https://sitemaps.org/protocol.html) for more details.
#[derive(Debug, PartialEq, Clone, Copy)]
pub struct UrlEntry<'a> {
    /// The location of this entry.
    ///
    /// `<loc>`
    ///
    /// I recommend using `http::Uri` to parse this, then extract the `Uri::path()`.
    pub location: &'a str,
    /// The date of last modification.
    ///
    /// `<lastmod>`
    ///
    /// Format should be in [W3C Datetime](https://www.w3.org/TR/NOTE-datetime).
    pub last_modified: Option<&'a str>,
    /// The frequency of change in this resource.
    ///
    /// `<changefreq>`
    pub change_frequency: Option<Frequency>,
    /// The priority of this page compared to other pages.
    ///
    /// `<priority>`
    ///
    /// Ranges from `0.0` to `1.0`
    pub priority: Option<f32>,
}
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Error {
    /// The mandatory `<urlset>` tag is missing.
    ///
    /// You maybe don't have a sitemap.
    UrlsetMissing,
    Parse(roxmltree::Error),
}
pub struct Document<'a> {
    doc: roxmltree::Document<'a>,
}
impl<'a> Document<'a> {
    /// Takes `xml_document` and parses it according to [the spec](https://sitemaps.org/protocol.html).
    pub fn parse(xml_document: &'a str) -> Result<Self, Error> {
        roxmltree::Document::parse(xml_document)
            .map_err(Error::Parse)
            .map(|doc| Self { doc })
    }
    /// Returns an iterator of [`UrlEntry`].
    ///
    /// Uses [`log`] for logging errors in the XML.
    pub fn iterate(
        &'a self,
    ) -> Result<impl Iterator<Item = UrlEntry<'a>> + DoubleEndedIterator + Clone + Debug + 'a, Error>
    {
        self.doc
            .root()
            .children()
            .find(|c| c.is_element())
            .and_then(|node| {
                if node.tag_name().name() == "urlset" {
                    Some(node)
                } else {
                    error!("Expected <urlset> but got {:?}", node);
                    None
                }
            })
            .map(|node| {
                node.children().filter_map(|c| {
                    let children = c.children().filter(|c| c.is_element());
                    let mut loc = None;
                    let mut lastmod = None;
                    let mut changefreq = None;
                    let mut priority = None;
                    for child in children {
                        if let Some(text) = node_text_expected_name(&child, "loc") {
                            if loc.is_none() {
                                loc = Some(text);
                            } else {
                                error!("Multiple <loc> in entry.");
                                return None;
                            }
                        } else if let Some(text) = node_text_expected_name(&child, "lastmod") {
                            if lastmod.is_some() {
                                warn!("Multiple <lastmod> in entry.");
                            }
                            lastmod = Some(text);
                        } else if let Some(text) = node_text_expected_name(&child, "changefreq") {
                            if changefreq.is_some() {
                                warn!("Multiple <changefreq> in entry.");
                            }
                            if let Ok(frequency) = text.parse() {
                                changefreq = Some(frequency);
                            } else {
                                warn!("<changefreq> has invalid format: {text:?}");
                            }
                        } else if let Some(text) = node_text_expected_name(&child, "priority") {
                            if priority.is_some() {
                                warn!("Multiple <priority> in entry.");
                            }
                            if let Ok(num) = text.parse() {
                                if (0.0..=1.0).contains(&num) {
                                    priority = Some(num)
                                } else {
                                    warn!("<priority> {num} is out of range",)
                                }
                            }else {
                                warn!("<priority> has invalid format: {text:?}. Expected floating-point number.");
                            }
                        }
                    }
                    if let Some(loc) = loc {
                        Some(UrlEntry::<'a> {
                            location: loc,
                            last_modified: lastmod,
                            change_frequency: changefreq,
                            priority,
                        })
                    } else {
                        error!("Expected <loc>, but found none.");
                        None
                    }
                })
            })
            .ok_or(Error::UrlsetMissing)
    }
}
fn node_text_expected_name<'a>(
    node: &roxmltree::Node<'a, 'a>,
    expected_tag: &str,
) -> Option<&'a str> {
    if node.tag_name().name() == expected_tag {
        if let Some(text) = node.text() {
            return Some(text);
        }
    }
    None
}
