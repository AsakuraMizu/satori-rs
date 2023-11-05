use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct At {
    #[serde(rename = "@id")]
    pub id: Option<String>,
    #[serde(rename = "@name")]
    pub name: Option<String>,
    #[serde(rename = "@role")]
    pub role: Option<String>,
    #[serde(rename = "@type")]
    pub ty: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Sharp {
    #[serde(rename = "@id")]
    pub id: String,
    #[serde(rename = "@name")]
    pub name: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Link {
    #[serde(rename = "@href")]
    pub href: String,
    #[serde(rename = "$value")]
    pub content: AnyMessage,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Img {
    #[serde(rename = "@src")]
    pub src: String,
    #[serde(rename = "@cache")]
    pub cache: bool,
    #[serde(rename = "@timeout")]
    pub timeout: String,
    #[serde(rename = "@width")]
    pub width: u32,
    #[serde(rename = "@height")]
    pub height: u32,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Audio {
    #[serde(rename = "@src")]
    pub src: String,
    #[serde(rename = "@cache")]
    pub cache: bool,
    #[serde(rename = "@timeout")]
    pub timeout: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Video {
    #[serde(rename = "@src")]
    pub src: String,
    #[serde(rename = "@cache")]
    pub cache: bool,
    #[serde(rename = "@timeout")]
    pub timeout: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct File {
    #[serde(rename = "@src")]
    pub src: String,
    #[serde(rename = "@cache")]
    pub cache: bool,
    #[serde(rename = "@timeout")]
    pub timeout: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Bold {
    #[serde(rename = "$value")]
    pub content: AnyMessage,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Italic {
    #[serde(rename = "$value")]
    pub content: AnyMessage,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Underline {
    #[serde(rename = "$value")]
    pub content: AnyMessage,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Strikethrough {
    #[serde(rename = "$value")]
    pub content: AnyMessage,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Spolier {
    #[serde(rename = "$value")]
    pub content: AnyMessage,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Code {
    #[serde(rename = "$value")]
    pub content: AnyMessage,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Superscript {
    #[serde(rename = "$value")]
    pub content: AnyMessage,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Subscript {
    #[serde(rename = "$value")]
    pub content: AnyMessage,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Paragraph {
    #[serde(rename = "$value")]
    pub content: AnyMessage,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Author {
    #[serde(rename = "@user-id")]
    pub user_id: Option<String>,
    #[serde(rename = "@nickname")]
    pub nickname: Option<String>,
    #[serde(rename = "@avatar")]
    pub avatar: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Message {
    #[serde(rename = "@id")]
    pub id: Option<String>,
    #[serde(rename = "@forward")]
    pub forward: Option<bool>,
    
    pub author: Option<Author>,

    #[serde(rename = "$value")]
    pub content: Option<AnyMessage>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Quote {
    #[serde(rename = "@id")]
    pub id: Option<String>,
    #[serde(rename = "$value")]
    pub content: Option<AnyMessage>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Element {
    At(At),
    Sharp(Sharp),
    A(Link),
    Img(Img),
    Audio(Audio),
    Video(Video),
    File(File),
    B(Bold),
    Strong(Bold),
    I(Italic),
    Em(Italic),
    U(Underline),
    Ins(Underline),
    S(Strikethrough),
    Del(Strikethrough),
    Spl(Spolier),
    Code(Code),
    Sup(Superscript),
    Sub(Subscript),
    Br,
    P(Paragraph),
    Message(Message),
    Quote(Quote),
    #[serde(rename = "$text")]
    Text(String),
}

pub type AnyMessage = Vec<Element>;
