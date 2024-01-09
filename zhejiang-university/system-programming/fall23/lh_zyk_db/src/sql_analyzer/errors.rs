use super::types::Span;
use miette::Diagnostic;
use nom_supreme::error::{BaseErrorKind, ErrorTree, GenericErrorTree, StackContext};
use thiserror::Error;

#[derive(Error, Debug, Diagnostic)]
#[error("Parse Error")]
pub struct FormattedError<'b> {
    // need 'b since Diagnostic derive uses 'a
    #[source_code]
    // marking what the passed string was
    pub src: &'b str,
    #[label("{kind}")]
    pub span: miette::SourceSpan,
    // tells miette to mark the provided span (the location in the
    // source code) with the error display from kind.
    pub kind: BaseErrorKind<&'b str, Box<dyn std::error::Error + Send + Sync + 'static>>,
    #[related]
    pub others: Vec<FormattedErrorContext<'b>>,
}

#[derive(Error, Debug, Diagnostic)]
#[error("Parse Error Context")]
pub struct FormattedErrorContext<'b> {
    #[source_code]
    pub src: &'b str,
    #[label("{context}")]
    pub span: miette::SourceSpan,
    pub context: StackContext<&'b str>,
}

pub type MyParseError<'a> = ErrorTree<Span<'a>>;

pub fn format_parse_error<'a>(input: &'a str, e: MyParseError<'a>) -> FormattedError<'a> {
    match e {
        // a "normal" error like unexpected charcter
        GenericErrorTree::Base { location, kind } => {
            // the location type is nom_locate's RawSpan type
            // Might be nice to just use our own span/make a wrapper to implement
            // From<OurSpan> for miette::SourceSpan
            let offset = location.location_offset().into();
            FormattedError {
                src: input,
                span: miette::SourceSpan::new(offset, 0.into()),
                kind,
                others: Vec::new(),
            }
        }
        // an error that has a context attached (from nom's context function)
        GenericErrorTree::Stack { base, contexts } => {
            let mut base = format_parse_error(input, *base);
            let mut contexts: Vec<FormattedErrorContext> = contexts
                .into_iter()
                .map(|(location, context)| {
                    let offset = location.location_offset().into();
                    FormattedErrorContext {
                        src: input,
                        span: miette::SourceSpan::new(offset, 0.into()),
                        context,
                    }
                })
                .collect();
            base.others.append(&mut contexts);
            base
        }
        // an error from an "alt"
        GenericErrorTree::Alt(alt_errors) => {
            // get the error with the most context
            // since that parsed the most stuff
            // TODO: figure out what to do on ties
            alt_errors
                .into_iter()
                .map(|e| format_parse_error(input, e))
                .max_by_key(|formatted| formatted.others.len())
                .unwrap()
        }
    }
}
