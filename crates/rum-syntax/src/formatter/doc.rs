use super::render::doc_trailing_state;

// The document IR forms the boundary between layout planning and final rendering.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum Doc {
    Text(String),
    #[allow(dead_code)]
    HardLine {
        indentation_level: usize,
    },
    SoftLine {
        indentation_level: usize,
    },
    Break {
        flat: String,
        indentation_level: usize,
    },
    Concat(Vec<Doc>),
    Group {
        doc: Box<Doc>,
        force_break: bool,
        flat_width: Option<usize>,
    },
}

#[derive(Debug)]
pub(super) struct DocBuilder {
    parts: Vec<Doc>,
    pending_break: Option<(String, usize)>,
    ends_with_newline: bool,
    ends_with_whitespace: bool,
}

impl DocBuilder {
    pub(super) fn new() -> Self {
        Self {
            parts: Vec::new(),
            pending_break: None,
            ends_with_newline: false,
            ends_with_whitespace: false,
        }
    }

    pub(super) fn is_empty(&self) -> bool {
        self.parts.is_empty()
    }

    pub(super) fn ends_with_newline(&self) -> bool {
        self.pending_break.is_some() || self.ends_with_newline
    }

    pub(super) fn ends_with_whitespace(&self) -> bool {
        self.pending_break.is_some() || self.ends_with_whitespace
    }

    pub(super) fn push_char(&mut self, ch: char) {
        self.push_text(&ch.to_string());
    }

    pub(super) fn push_text(&mut self, text: &str) {
        if text.is_empty() {
            return;
        }

        self.flush_pending_break();

        match self.parts.last_mut() {
            Some(Doc::Text(existing)) => existing.push_str(text),
            _ => self.parts.push(Doc::Text(text.to_owned())),
        }

        if let Some(last_char) = text.chars().next_back() {
            self.ends_with_newline = last_char == '\n';
            self.ends_with_whitespace = last_char.is_whitespace();
        }
    }

    pub(super) fn push_doc(&mut self, doc: Doc) {
        self.flush_pending_break();

        match doc {
            Doc::Text(text) => self.push_text(&text),
            other => {
                let (ends_with_newline, ends_with_whitespace) = doc_trailing_state(&other);
                self.parts.push(other);
                self.ends_with_newline = ends_with_newline;
                self.ends_with_whitespace = ends_with_whitespace;
            }
        }
    }

    pub(super) fn push_break(&mut self, flat: String, indentation_level: usize) {
        if self.ends_with_newline() {
            return;
        }

        self.pending_break = Some((flat, indentation_level));
        self.ends_with_newline = true;
        self.ends_with_whitespace = true;
    }

    pub(super) fn finish(mut self) -> Doc {
        self.flush_pending_break();
        concat_docs(self.parts)
    }

    fn flush_pending_break(&mut self) {
        if let Some((flat, indentation_level)) = self.pending_break.take() {
            self.parts.push(Doc::Break {
                flat,
                indentation_level,
            });
        }
    }
}

fn concat_docs(parts: Vec<Doc>) -> Doc {
    if parts.len() == 1 {
        parts
            .into_iter()
            .next()
            .expect("single-part document should exist")
    } else {
        Doc::Concat(parts)
    }
}

pub(super) fn append_softline(doc: Doc, indentation_level: usize) -> Doc {
    if doc_is_empty(&doc) || doc_trailing_state(&doc).0 {
        return doc;
    }

    concat_docs(vec![doc, Doc::SoftLine { indentation_level }])
}

fn doc_is_empty(doc: &Doc) -> bool {
    matches!(doc, Doc::Concat(parts) if parts.is_empty())
}
