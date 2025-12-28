pub struct Line {
    pub kind: LineKind,
    pub content: Option<String>,
    pub comment: Option<Comment>,
}

pub struct Comment {
    pub delimiter: Delimiter,
    pub content: String,
}

pub enum LineKind {
    Date,
    Posting,
    Comment,
    Other,
    None,
}

pub enum Delimiter {
    Hash,
    Semicolon,
    None,
}

impl std::fmt::Display for Comment {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}{}",
            match self.delimiter {
                Delimiter::Hash => "#",
                Delimiter::Semicolon => ";",
                Delimiter::None => "",
            },
            self.content
        )
    }
}
