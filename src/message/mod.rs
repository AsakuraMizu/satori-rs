mod elements;
pub use elements::*;

pub fn from_str(s: &str) -> Result<elements::AnyMessage, quick_xml::DeError> {
    quick_xml::de::from_str(s)
}

pub fn to_string(msg: &elements::AnyMessage) -> Result<String, quick_xml::DeError> {
    msg.iter()
        .map(|m| quick_xml::se::to_string(m))
        .collect::<Result<Vec<_>, _>>()
        .map(|r| r.join(""))
}
