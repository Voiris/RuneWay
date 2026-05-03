use std::fmt::Write;
use indexmap::IndexMap;
use runec_source::source_map::{SourceId, SourceMap};
use runec_utils::common::number_length::number_length;
use crate::diagnostics::{DiagType, Diagnostic};
use crate::labels::{DiagHelp, DiagLabel, DiagNote};
use crate::message::DiagMessage;

impl<'diag> Diagnostic<'diag> {
    #[doc(hidden)]
    fn write_emit_header(diag_type: DiagType, diag_code: Option<u16>, message: DiagMessage, out: &mut impl Write) {
        write!(out, "{}", diag_type).unwrap();
        if let Some(code) = diag_code {
            write!(
                out,
                "\x1b[1;36m[E{:04}]\x1b[0m",
                code,
            ).unwrap();
        }
        write!(
            out,
            ": {}",
            message.message
        ).unwrap();
    }

    #[doc(hidden)]
    fn group_labels_by_source(labels: Vec<DiagLabel>) -> IndexMap<SourceId, Vec<DiagLabel>> {
        let mut source_labels = IndexMap::<SourceId, Vec<DiagLabel>>::new();
        for label in labels {
            source_labels
                .entry(label.span.src_id)
                .or_default()
                .push(label);
        }
        source_labels
    }

    #[doc(hidden)]
    fn calculate_separator_offset(
        source_labels: &IndexMap<SourceId, Vec<DiagLabel>>,
        source_map: &SourceMap,
    ) -> usize {
        let max_line_number = source_labels.keys().map(
            |id| source_map
                .get_file(id).unwrap()
                .lines().last_line_number()
                .to_usize() + 1
        ).max().unwrap();
        number_length(max_line_number) + 1
    }

    #[doc(hidden)]
    fn write_emit_labels(
        source_map: &SourceMap,
        separator_offset: usize,
        source_labels: IndexMap<SourceId, Vec<DiagLabel>>,
        out: &mut impl Write
    ) -> usize {
        for (source_id, labels) in source_labels {
            let source_file = source_map.get_file(&source_id).unwrap();
            write!(
                out,
                "\n{}\x1b[1;96m-->\x1b[0m {}",
                " ".repeat(separator_offset - 1),
                source_file.path().display()
            ).unwrap();
            let source_text = source_file.src();
            if !labels.is_empty() {
                write!(out, "\n\x1b[1;96m{}|", " ".repeat(separator_offset)).unwrap()
            }
            for label in labels {
                let (line, line_start) = {
                    let (line_idx, line_start) = source_file.lines().line_search(label.span.lo);
                    (line_idx.to_usize() + 1, line_start.to_usize())
                };
                let line_end = line_start + source_text[line_start..].chars().position(|c| c == '\n').unwrap_or(source_text.len());
                let line_text = &source_text[line_start..line_end];
                let text_marker_offset = line_text
                    .chars()
                    .take(label.span.lo.to_usize() - line_start)
                    .map(|c| if c == '\t' { 4 } else { 1 })
                    .sum::<usize>();
                let marker_len = source_text[label.span.lo.to_usize()..label.span.hi.to_usize()].chars().count();
                write!(
                    out,
                    "\n\x1b[1;96m{}{}|\x1b[0m {}\n{}\x1b[1;96m| {}{}{}\x1b[0m",
                    line,
                    " ".repeat(separator_offset - number_length(line)),
                    line_text,
                    " ".repeat(separator_offset),
                    " ".repeat(text_marker_offset),
                    label.kind.color_code(),
                    label.kind.marker().to_string().repeat(marker_len),
                ).unwrap();
                if let Some(label_message) = label.message {
                    write!(
                        out,
                        " {}{}\x1b[0m",
                        label.kind.color_code(),
                        label_message
                    ).unwrap();
                }
            }
        };

        separator_offset
    }

    #[doc(hidden)]
    fn write_emit_sublabel(
        separator_offset: usize,
        message: String,
        sublabel_type: &'static str,
        out: &mut impl Write,
    ) {
        write!(
            out,
            "\n{}\x1b[96m= \x1b[97m{}\x1b[0m: {}",
            " ".repeat(separator_offset),
            sublabel_type,
            message
        ).unwrap();
    }

    #[doc(hidden)]
    fn write_emit_sublabels(
        separator_offset: usize,
        help_opt: Option<DiagHelp>,
        note_opt: Option<DiagNote>,
        out: &mut impl Write,
    ) {
        if let Some(help) = help_opt {
            Self::write_emit_sublabel(
                separator_offset,
                help.message,
                "help",
                out,
            );
        }
        if let Some(note) = note_opt {
            Self::write_emit_sublabel(
                separator_offset,
                note.message,
                "note",
                out,
            );
        }
    }

    /// Emits a diagnostic to the given output buffer
    // Diagnostic formatting design inspired by Rust compiler: https://github.com/rust-lang/rust
    pub fn emit(self, source_map: &SourceMap, out: &mut impl Write) {
        Self::write_emit_header(self.diag_type, self.code, self.message, out);
        let source_labels = Self::group_labels_by_source(self.labels);
        let separator_offset = Self::calculate_separator_offset(
            &source_labels,
            source_map
        );
        Self::write_emit_labels(
            source_map,
            separator_offset,
            source_labels,
            out
        );
        Self::write_emit_sublabels(separator_offset, self.help, self.note, out);
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use runec_source::byte_pos::BytePos;
    use runec_source::span::Span;

    use crate::labels::{DiagHelp, DiagNote};
    use crate::message::DiagMessage;
    use super::*;

    use runec_test_utils::MockSourceFileLoader;

    #[test]
    fn test_emit() {
        let mut source_map = SourceMap::new();
        let path = PathBuf::from("/home/user/main.rnw");
        let source = "01234567\n8\n\n\t987654321\n";
        let mock = MockSourceFileLoader { source };
        let source_id = source_map.add_file(
            mock.load(path).unwrap()
        );
        let diagnostic = Diagnostic::error_with_code(
            DiagMessage::new("void {msg}", &[("msg", "message")]),
            102
        )
            .add_label(
                DiagLabel::primary("void {msg}", &[("msg", "message")], Span::new(
                    BytePos::from_usize(1),
                    BytePos::from_usize(5),
                    source_id,
                ))
            )
            .add_label(
                DiagLabel::silent_secondary(Span::new(
                    BytePos::from_usize(16),
                    BytePos::from_usize(19),
                    source_id,
                ))
            )
            .set_note(
                DiagNote::new("void {msg}", &[("msg", "message")])
            )
            .set_help(
                DiagHelp::new("void {msg}", &[("msg", "message")])
            );

        let mut buffer = String::new();
        diagnostic.emit(&source_map, &mut buffer);

        assert_eq!(buffer,
                   "\x1b[1;91merror\x1b[0m\x1b[1;36m[E0102]\x1b[0m: void message\n \x1b[1;96m-->\x1b[0m /home/user/main.rnw\n\x1b[1;96m  |\
                   \n\x1b[1;96m1 |\x1b[0m 01234567\n  \x1b[1;96m|  \x1b[1;96m^^^^\x1b[0m \x1b[1;96mvoid message\
                   \x1b[0m\n\x1b[1;96m4 |\x1b[0m \t987654321\n  \x1b[1;96m|        \x1b[1;93m---\x1b[0m\n  \x1b[96m= \
                   \x1b[97mhelp\x1b[0m: void message\n  \x1b[96m= \x1b[97mnote\x1b[0m: void message"
        );
    }
}
