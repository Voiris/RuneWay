use std::collections::HashMap;
use std::fmt::Write;
use fluent::{FluentArgs, FluentValue};
use indexmap::IndexMap;
use runec_fluent_messages::{get_fluent_bundle, get_fluent_message};
use runec_source::source_map::{SourceId, SourceMap};
use runec_utils::common::number_length::number_length;
use crate::diagnostics::{DiagType, Diagnostic};
use crate::labels::{DiagHelp, DiagLabel, DiagNote};
use crate::message::DiagMessage;

impl<'a> Diagnostic<'a> {
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
        let bundle = get_fluent_bundle();
        {
            let message_args = message.args.map(FluentArgs::from_iter);
            let mut err = Vec::new();
            write!(
                out,
                ": {}",
                bundle.format_pattern(
                    get_fluent_message(message.message_id).value().unwrap(),
                    message_args.as_ref(),
                    &mut err
                )
            ).unwrap();
        }
    }

    #[doc(hidden)]
    fn group_labels_by_source(labels: Vec<DiagLabel<'a>>) -> IndexMap<SourceId, Vec<DiagLabel<'a>>> {
        let mut source_labels = IndexMap::<SourceId, Vec<DiagLabel<'a>>>::new();
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
        source_labels: &IndexMap<SourceId, Vec<DiagLabel<'a>>>,
        source_map: &SourceMap,
    ) -> usize {
        let max_line_number = source_labels.keys().map(
            |id| source_map
                .get_file(id).unwrap()
                .lines.last_line_number()
                .to_usize() + 1
        ).max().unwrap();
        number_length(max_line_number) + 1
    }

    #[doc(hidden)]
    fn write_emit_labels(
        source_map: &SourceMap,
        separator_offset: usize,
        source_labels: IndexMap<SourceId, Vec<DiagLabel<'a>>>,
        out: &mut impl Write
    ) -> usize {
        let bundle = get_fluent_bundle();
        for (source_id, labels) in source_labels {
            let source_file = source_map.get_file(&source_id).unwrap();
            write!(
                out,
                "\n{}\x1b[1;96m-->\x1b[0m {}",
                " ".repeat(separator_offset - 1),
                source_file.file_name
            ).unwrap();
            let source_text = source_file.src.as_ref();
            if !labels.is_empty() {
                write!(out, "\n\x1b[1;96m{}|", " ".repeat(separator_offset)).unwrap()
            }
            for label in labels {
                let (line, line_start) = {
                    let (line_idx, line_start) = source_file.lines.line_search(label.span.lo);
                    (line_idx.to_usize() + 1, line_start.to_usize())
                };
                let line_end = line_start + source_text[line_start..].chars().position(|c| c == '\n').unwrap_or(source_text.len());
                let line_text = &source_text[line_start..line_end];
                let text_marker_offset = line_text
                    .chars()
                    .take(label.span.lo.to_usize() - line_start)
                    .map(|c| if c == '\t' { 4 } else { 1 })
                    .sum::<usize>();
                write!(
                    out,
                    "\n\x1b[1;96m{}{}|\x1b[0m {}\n{}\x1b[1;96m| {}{}{}\x1b[0m",
                    line,
                    " ".repeat(separator_offset - number_length(line)),
                    line_text,
                    " ".repeat(separator_offset),
                    " ".repeat(text_marker_offset),
                    label.kind.color_code(),
                    label.kind.marker().to_string().repeat(label.span.hi.to_usize() - label.span.lo.to_usize()),
                ).unwrap();
                if let Some(label_message_id) = label.message_id {
                    let label_args = label.args.map(FluentArgs::from_iter);
                    let mut err = Vec::new();
                    write!(
                        out,
                        " {}{}\x1b[0m",
                        label.kind.color_code(),
                        bundle.format_pattern(
                            get_fluent_message(label_message_id).value().unwrap(),
                            label_args.as_ref(),
                            &mut err
                        )
                    ).unwrap();
                }
            }
        };

        separator_offset
    }

    #[doc(hidden)]
    fn write_emit_sublabel(
        separator_offset: usize,
        message_id: &'static str,
        message_args: Option<HashMap<&'static str, FluentValue<'a>>>,
        sublabel_type: &'static str,
        out: &mut impl Write,
    ) {
        let label_args = message_args.map(FluentArgs::from_iter);
        let mut err = Vec::new();
        write!(
            out,
            "\n{}\x1b[96m= \x1b[97m{}\x1b[0m: {}",
            " ".repeat(separator_offset),
            sublabel_type,
            get_fluent_bundle().format_pattern(
                get_fluent_message(message_id).value().unwrap(),
                label_args.as_ref(),
                &mut err
            )
        ).unwrap();
    }

    #[doc(hidden)]
    fn write_emit_sublabels(
        separator_offset: usize,
        help_opt: Option<DiagHelp<'a>>,
        note_opt: Option<DiagNote<'a>>,
        out: &mut impl Write,
    ) {
        if let Some(help) = help_opt {
            Self::write_emit_sublabel(
                separator_offset,
                help.message_id,
                help.args,
                "help",
                out,
            );
        }
        if let Some(note) = note_opt {
            Self::write_emit_sublabel(
                separator_offset,
                note.message_id,
                note.args,
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
    use runec_source::source_map::{FileName, SourceFile};
    use runec_source::span::Span;
    use crate::labels::{DiagHelp, DiagNote};
    use crate::message::DiagMessage;
    use super::*;

    #[test]
    fn test_emit() {
        let mut source_map = SourceMap::new();
        let source_id = source_map.add_file(
            SourceFile::new(FileName::Real(PathBuf::from("/home/user/main.rnw")), "01234567\n8\n\n\t987654321\n".to_owned())
        );
        let diagnostic = Diagnostic::error_with_code(
            DiagMessage::new_simple("void"),
            102
        )
            .add_label(
                DiagLabel::simple_primary("void", Span::new(
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
                DiagNote::new_simple("void")
            )
            .set_help(
                DiagHelp::new_simple("void")
            );

        let mut buffer = String::new();
        diagnostic.emit(&source_map, &mut buffer);

        println!("{}", buffer);

        assert_eq!(buffer,
                   "\x1b[1;91merror\x1b[0m\x1b[1;36m[E0102]\x1b[0m: void message\n \x1b[1;96m-->\x1b[0m /home/user/main.rnw\n\x1b[1;96m  |\
                   \n\x1b[1;96m1 |\x1b[0m 01234567\n  \x1b[1;96m|  \x1b[1;96m----\x1b[0m \x1b[1;96mvoid message\
                   \x1b[0m\n\x1b[1;96m4 |\x1b[0m \t987654321\n  \x1b[1;96m|        \x1b[1;93m^^^\x1b[0m\n  \x1b[96m= \
                   \x1b[97mhelp\x1b[0m: void message\n  \x1b[96m= \x1b[97mnote\x1b[0m: void message"
        );
    }
}
