use runec_source::span::Span;

#[derive(Debug)]
pub enum DiagLabelKind {
    Primary,
    Secondary,
}

impl DiagLabelKind {
    pub fn marker(&self) -> char {
        match self {
            DiagLabelKind::Primary => '^',
            DiagLabelKind::Secondary => '-',
        }
    }

    pub fn color_code(&self) -> &'static str {
        match self {
            DiagLabelKind::Primary => "\x1b[1;96m",
            DiagLabelKind::Secondary => "\x1b[1;93m",
        }
    }
}

#[derive(Debug)]
pub struct DiagLabel {
    pub message: Option<String>,
    pub kind: DiagLabelKind,
    pub span: Span
}

impl DiagLabel {
    pub fn new(message: Option<&'static str>, args: &[(&str, &str)], kind: DiagLabelKind, span: Span) -> Self {
        Self {
            message: message.map(|m| runec_utils::common::message_format::message_format(m, args)),
            kind,
            span,
        }
    }

    pub fn primary(message: &'static str, args: &[(&str, &str)], span: Span) -> Self {
        Self::new(Some(message), args, DiagLabelKind::Primary, span)
    }

    pub fn simple_primary(message: &'static str, span: Span) -> Self {
        Self::new(Some(message), &[], DiagLabelKind::Primary, span)
    }

    pub fn silent_primary(span: Span) -> Self {
        Self::new(None, &[], DiagLabelKind::Primary, span)
    }

    pub fn secondary(message: Option<&'static str>, args: &[(&str, &str)], span: Span) -> Self {
        Self::new(message, args, DiagLabelKind::Secondary, span)
    }

    pub fn simple_secondary(message: &'static str, span: Span) -> Self {
        Self::new(Some(message), &[], DiagLabelKind::Secondary, span)
    }

    pub fn silent_secondary(span: Span) -> Self {
        Self::new(None, &[], DiagLabelKind::Secondary, span)
    }
}

#[derive(Debug)]
pub struct DiagNote {
    pub message: String,
}

impl DiagNote {
    impl_message_new!();
}

#[derive(Debug)]
pub struct DiagHelp<> {
    pub message: String,
}

impl<> DiagHelp<> {
    impl_message_new!();
}
