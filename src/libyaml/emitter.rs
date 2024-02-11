use super::Error;
use libyaml_safer as sys;
use std::io;

pub(crate) struct Emitter<'a> {
    // Note: Lifetime is actually "'self". Is drop order technically significant?
    sys: sys::Emitter<'a>,
    write: Box<dyn io::Write + 'a>,
    _pin: std::marker::PhantomPinned,
}

#[derive(Debug)]
pub(crate) enum Event<'a> {
    StreamStart,
    StreamEnd,
    DocumentStart,
    DocumentEnd,
    Scalar(Scalar<'a>),
    SequenceStart(Sequence),
    SequenceEnd,
    MappingStart(Mapping),
    MappingEnd,
}

#[derive(Debug)]
pub(crate) struct Scalar<'a> {
    pub tag: Option<String>,
    pub value: &'a str,
    pub style: ScalarStyle,
}

#[derive(Debug)]
pub(crate) enum ScalarStyle {
    Any,
    Plain,
    SingleQuoted,
    Literal,
}

#[derive(Debug)]
pub(crate) struct Sequence {
    pub tag: Option<String>,
}

#[derive(Debug)]
pub(crate) struct Mapping {
    pub tag: Option<String>,
}

impl<'a> Emitter<'a> {
    pub fn new(mut write: Box<dyn io::Write + 'a>) -> Emitter<'a> {
        let mut sys = sys::Emitter::new();
        sys.set_unicode(true);
        sys.set_width(-1);
        let writer = &mut *write as *mut dyn io::Write;
        sys.set_output(unsafe {
            // Upcast lifetime
            &mut *writer
        });
        let emitter = Emitter {
            sys,
            write,
            _pin: std::marker::PhantomPinned,
        };
        emitter
    }

    pub fn emit(&mut self, event: Event) -> Result<(), Error> {
        let event = match event {
            Event::StreamStart => sys::Event::stream_start(sys::Encoding::Utf8),
            Event::StreamEnd => sys::Event::stream_end(),
            Event::DocumentStart => sys::Event::document_start(None, &[], true),
            Event::DocumentEnd => sys::Event::document_end(true),
            Event::Scalar(scalar) => sys::Event::scalar(
                None,
                scalar.tag.as_deref(),
                scalar.value,
                scalar.tag.is_none(),
                scalar.tag.is_none(),
                match scalar.style {
                    ScalarStyle::Any => sys::ScalarStyle::Any,
                    ScalarStyle::Plain => sys::ScalarStyle::Plain,
                    ScalarStyle::SingleQuoted => sys::ScalarStyle::SingleQuoted,
                    ScalarStyle::Literal => sys::ScalarStyle::Literal,
                },
            ),
            Event::SequenceStart(seq) => sys::Event::sequence_start(
                None,
                seq.tag.as_deref(),
                seq.tag.is_none(),
                sys::SequenceStyle::Any,
            ),
            Event::SequenceEnd => sys::Event::sequence_end(),
            Event::MappingStart(mapping) => sys::Event::mapping_start(
                None,
                mapping.tag.as_deref(),
                mapping.tag.is_none(),
                sys::MappingStyle::Any,
            ),
            Event::MappingEnd => sys::Event::mapping_end(),
        };

        self.sys.emit(event)?;
        Ok(())
    }

    pub fn flush(&mut self) -> Result<(), Error> {
        self.sys.flush()?;
        Ok(())
    }

    pub fn into_inner(self) -> Box<dyn io::Write + 'a> {
        self.write
    }
}
