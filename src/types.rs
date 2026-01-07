pub struct Line {
    pub kind: LineKind,
    pub content: Option<String>,
    pub comment: Option<Comment>,
}

pub struct Comment {
    pub delimiter: Delimiter,
    pub content: String,
}

pub struct Amount {
    pub amount: f64,
    pub precision: usize,
    pub currency: Option<Currency>,
}

pub struct Currency {
    pub symbol: String,
    pub prepend: bool,
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

impl std::fmt::Display for Amount {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut out: String = format!("{:.*}", self.precision, self.amount);
        if let Some(currency) = &self.currency {
            match currency.prepend {
                true => {
                    if self.amount > 0.0 {
                        out.insert(0, ' ');
                    }
                    out.insert_str(0, &currency.symbol);
                }
                false => out.push_str(&currency.symbol),
            }
        }
        write!(f, "{}", out)
    }
}
