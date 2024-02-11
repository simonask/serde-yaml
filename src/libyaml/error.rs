use libyaml_safer as sys;
use std::fmt::{self, Debug, Display};

pub(crate) type Result<T> = std::result::Result<T, Error>;

pub(crate) struct Error {
    sys: sys::Error,
}

impl Error {
    pub fn mark(&self) -> Option<Mark> {
        Some(Mark {
            sys: self.sys.problem_mark()?,
        })
    }

    fn context_mark(&self) -> Option<Mark> {
        Some(Mark {
            sys: self.sys.context_mark()?,
        })
    }
}

impl From<sys::Error> for Error {
    fn from(value: sys::Error) -> Self {
        Error { sys: value }
    }
}

impl Display for Error {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "{}", self.sys.problem())?;
        if let Some(problem_mark) = self.mark() {
            if problem_mark.sys.line != 0 || problem_mark.sys.column != 0 {
                write!(formatter, " at {}", problem_mark)?;
            } else if problem_mark.index() != 0 {
                write!(formatter, " at position {}", problem_mark.index())?;
            }
        }
        if let Some(context) = self.sys.context() {
            write!(formatter, ", {}", context)?;
            if let Some(context_mark) = self.context_mark() {
                if (context_mark.sys.line != 0 || context_mark.sys.column != 0)
                    && Some(context_mark) != self.mark()
                {
                    write!(formatter, " at {}", context_mark)?;
                }
            }
        }
        Ok(())
    }
}

impl Debug for Error {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        let mut formatter = formatter.debug_struct("Error");
        if let Some(kind) = match self.sys.kind() {
            sys::ErrorKind::Reader => Some("READER"),
            sys::ErrorKind::Scanner => Some("SCANNER"),
            sys::ErrorKind::Parser => Some("PARSER"),
            sys::ErrorKind::Composer => Some("COMPOSER"),
            sys::ErrorKind::Emitter => Some("EMITTER"),
            _ => None,
        } {
            formatter.field("kind", &format_args!("{}", kind));
        }
        formatter.field("problem", &self.sys.problem());
        if let Some(problem_mark) = self.mark() {
            if problem_mark.sys.line != 0 || problem_mark.sys.column != 0 {
                formatter.field("problem_mark", &problem_mark);
            } else if problem_mark.sys.index != 0 {
                formatter.field("problem_offset", &problem_mark.sys.index);
            }
        }
        if let Some(context) = self.sys.context() {
            formatter.field("context", &context);
            if let Some(context_mark) = self.context_mark() {
                if context_mark.sys.line != 0 || context_mark.sys.column != 0 {
                    formatter.field("context_mark", &context_mark);
                }
            }
        }
        formatter.finish()
    }
}

#[derive(Copy, Clone, PartialEq)]
pub(crate) struct Mark {
    pub(super) sys: sys::Mark,
}

impl Mark {
    pub fn index(&self) -> u64 {
        self.sys.index
    }

    pub fn line(&self) -> u64 {
        self.sys.line
    }

    pub fn column(&self) -> u64 {
        self.sys.column
    }
}

impl Display for Mark {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        if self.sys.line != 0 || self.sys.column != 0 {
            write!(
                formatter,
                "line {} column {}",
                self.sys.line + 1,
                self.sys.column + 1,
            )
        } else {
            write!(formatter, "position {}", self.sys.index)
        }
    }
}

impl Debug for Mark {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        let mut formatter = formatter.debug_struct("Mark");
        if self.sys.line != 0 || self.sys.column != 0 {
            formatter.field("line", &(self.sys.line + 1));
            formatter.field("column", &(self.sys.column + 1));
        } else {
            formatter.field("index", &self.sys.index);
        }
        formatter.finish()
    }
}
