macro_rules! impl_add_arg {
    ($lifetime:lifetime) => {
        pub fn add_arg(mut self, key: &'static str, arg: FluentValue<$lifetime>) -> Self {
            self.args.get_or_insert_with(HashMap::new).insert(key, arg);
            self
        }
    }
}

macro_rules! impl_message_new {
    ($lifetime:lifetime) => {
        pub fn new(message_id: &'static str, args: Option<HashMap<&'static str, FluentValue<$lifetime>>>) -> Self {
            Self { message_id, args }
        }

        pub fn new_simple(message_id: &'static str) -> Self {
            Self::new(message_id, None)
        }
    }
}

#[macro_export]
macro_rules! make_simple_diag {
    (error; $message_id:literal $(, $rest:tt )* $(,)?) => {{
        let diag = $crate::diagnostics::Diagnostic::error($crate::message::DiagMessage::new_simple($message_id));
        $crate::make_simple_diag!(@internal diag; $( $rest, )*)
    }};
    (error : $code:expr; $message_id:literal $(, $rest:tt )* $(,)?) => {{
        let diag = $crate::diagnostics::Diagnostic::error_with_code($crate::message::DiagMessage::new_simple($message_id), $code);
        $crate::make_simple_diag!(@internal diag; $( $rest, )*)
    }};
    (warning; $message_id:literal  $(, $rest:tt )* $(,)?) => {{
        let diag = $crate::diagnostics::Diagnostic::warning($crate::message::DiagMessage::new_simple($message_id));
        $crate::make_simple_diag!(@internal diag; $( $rest, )*)
    }};
    (weak_warning; $message_id:literal  $(, $rest:tt )* $(,)?) => {{
        let diag = ;$crate::diagnostics::Diagnostic::warning($crate::message::DiagMessage::new_simple($message_id));
        $crate::make_simple_diag!(@internal diag; $( $rest, )*)
    }};
    (
        @internal
        $diag:expr;
        $( ($( : $primary_label:literal   : )? $primary_source_id:expr   => $primary_span_range:expr),   )?
        $( [$( : $secondary_label:literal : )? $secondary_source_id:expr => $secondary_span_range:expr], )*
        $( {help = $help_message_id:literal}, )?
        $( {note = $note_message_id:literal}  )?
        $(,)?
    ) => {{
        $diag
            $(
            .add_label(
                $crate::make_simple_diag!(@internal_primary_label $( $primary_label : )? $primary_source_id => $primary_span_range )
            )
            )?
            $(
            .add_label(
                $crate::make_simple_diag!(@internal_secondary_label $( $secondary_label : )? $secondary_source_id => $secondary_span_range )
            )
            )*
            $(
            .set_help($crate::labels::DiagHelp::new_simple(
                $help_message_id
            ))
            )?
            $(
            .set_note($crate::labels::DiagNote::new_simple(
                $note_message_id
            ))
            )?
    }};
    (
        @internal_primary_label
        $primary_label:literal :
        $primary_source_id:expr => $primary_span_range:expr
    ) => {
        $crate::labels::DiagLabel::simple_primary(
            $primary_label,
            runec_source::span::Span::new($primary_span_range.start, $primary_span_range.end, $primary_source_id),
        )
    };
    (
        @internal_primary_label
        $primary_source_id:expr => $primary_span_range:expr
    ) => {
        $crate::labels::DiagLabel::silent_primary(
            runec_source::span::Span::new($primary_span_range.start, $primary_span_range.end, $primary_source_id),
        )
    };
    (
        @internal_secondary_label
        $secondary_label:literal :
        $secondary_source_id:expr => $secondary_span_range:expr
    ) => {
        $crate::labels::DiagLabel::simple_secondary(
            $secondary_label,
            runec_source::span::Span::new($secondary_span_range.start, $secondary_span_range.end, $secondary_source_id),
        )
    };
    (
        @internal_secondary_label
        $secondary_source_id:expr => $secondary_span_range:expr
    ) => {
        $crate::labels::DiagLabel::silent_secondary(
            runec_source::span::Span::new($secondary_span_range.start, $secondary_span_range.end, $secondary_source_id),
        )
    };
}
