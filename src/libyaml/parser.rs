use crate::libyaml::cstr::{self};
use crate::libyaml::error::{Mark, Result};
use crate::libyaml::tag::Tag;
use libyaml_safer as sys;
use std::borrow::Cow;
use std::fmt::{self, Debug};

pub(crate) struct Parser<'input> {
    // Note: Lifetime is actually "'self".
    sys: sys::Parser<'input>,
    input: Cow<'input, [u8]>,
    _input_cursor: Box<&'input [u8]>,
    _pin: std::marker::PhantomPinned,
}

#[derive(Debug)]
pub(crate) enum Event<'input> {
    StreamStart,
    StreamEnd,
    DocumentStart,
    DocumentEnd,
    Alias(Anchor),
    Scalar(Scalar<'input>),
    SequenceStart(SequenceStart),
    SequenceEnd,
    MappingStart(MappingStart),
    MappingEnd,
}

pub(crate) struct Scalar<'input> {
    pub anchor: Option<Anchor>,
    pub tag: Option<Tag>,
    pub value: Box<[u8]>,
    pub style: ScalarStyle,
    pub repr: Option<&'input [u8]>,
}

#[derive(Debug)]
pub(crate) struct SequenceStart {
    pub anchor: Option<Anchor>,
    pub tag: Option<Tag>,
}

#[derive(Debug)]
pub(crate) struct MappingStart {
    pub anchor: Option<Anchor>,
    pub tag: Option<Tag>,
}

#[derive(Ord, PartialOrd, Eq, PartialEq)]
pub(crate) struct Anchor(Box<[u8]>);

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub(crate) enum ScalarStyle {
    Plain,
    SingleQuoted,
    DoubleQuoted,
    Literal,
    Folded,
}

impl<'input> Parser<'input> {
    pub fn new(input: Cow<'input, [u8]>) -> Parser<'input> {
        let mut sys = sys::Parser::new();
        let slice = &*input as *const [u8];
        let mut cursor = unsafe {
            // Upcast lifetime
            Box::new(&*slice as &'input [u8])
        };
        let cursor_ptr = &mut *cursor as *mut &'input [u8];
        sys.set_input(unsafe {
            // Upcast lifetime
            &mut *cursor_ptr
        });
        sys.set_encoding(sys::Encoding::Utf8);
        Self {
            sys,
            input,
            _input_cursor: cursor,
            _pin: std::marker::PhantomPinned,
        }
    }

    pub fn next(&mut self) -> Result<(Event<'input>, Mark)> {
        let event = self.sys.parse()?;
        let mark = event.start_mark;
        Ok((convert_event(event, &self.input), Mark { sys: mark }))
    }
}

fn convert_event<'input>(sys: sys::Event, input: &Cow<'input, [u8]>) -> Event<'input> {
    match sys.data {
        sys::EventData::StreamStart { .. } => Event::StreamStart,
        sys::EventData::StreamEnd => Event::StreamEnd,
        sys::EventData::DocumentStart { .. } => Event::DocumentStart,
        sys::EventData::DocumentEnd { .. } => Event::DocumentEnd,
        sys::EventData::Alias { anchor } => {
            Event::Alias(Anchor(anchor.into_boxed_str().into_boxed_bytes()))
        }
        sys::EventData::Scalar {
            anchor,
            tag,
            value,
            style,
            ..
        } => Event::Scalar(Scalar {
            anchor: anchor.map(|anchor| Anchor(anchor.into_boxed_str().into_boxed_bytes())),
            tag: tag.map(|tag| Tag(tag.into_boxed_str().into_boxed_bytes())),
            value: value.into_boxed_str().into_boxed_bytes(),
            style: match style {
                sys::ScalarStyle::Plain => ScalarStyle::Plain,
                sys::ScalarStyle::SingleQuoted => ScalarStyle::SingleQuoted,
                sys::ScalarStyle::DoubleQuoted => ScalarStyle::DoubleQuoted,
                sys::ScalarStyle::Literal => ScalarStyle::Literal,
                sys::ScalarStyle::Folded => ScalarStyle::Folded,
                sys::ScalarStyle::Any | _ => unreachable!(),
            },
            repr: if let Cow::Borrowed(input) = input {
                Some(&input[sys.start_mark.index as usize..sys.end_mark.index as usize])
            } else {
                None
            },
        }),
        sys::EventData::SequenceStart { anchor, tag, .. } => Event::SequenceStart(SequenceStart {
            anchor: anchor.map(|anchor| Anchor(anchor.into_boxed_str().into_boxed_bytes())),
            tag: tag.map(|tag| Tag(tag.into_boxed_str().into_boxed_bytes())),
        }),
        sys::EventData::SequenceEnd => Event::SequenceEnd,
        sys::EventData::MappingStart { anchor, tag, .. } => Event::MappingStart(MappingStart {
            anchor: anchor.map(|anchor| Anchor(anchor.into_boxed_str().into_boxed_bytes())),
            tag: tag.map(|tag| Tag(tag.into_boxed_str().into_boxed_bytes())),
        }),
        sys::EventData::MappingEnd => Event::MappingEnd,
    }
}

impl<'input> Debug for Scalar<'input> {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        let Scalar {
            anchor,
            tag,
            value,
            style,
            repr: _,
        } = self;

        struct LossySlice<'a>(&'a [u8]);

        impl<'a> Debug for LossySlice<'a> {
            fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                cstr::debug_lossy(self.0, formatter)
            }
        }

        formatter
            .debug_struct("Scalar")
            .field("anchor", anchor)
            .field("tag", tag)
            .field("value", &LossySlice(value))
            .field("style", style)
            .finish()
    }
}

impl Debug for Anchor {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        cstr::debug_lossy(&self.0, formatter)
    }
}
