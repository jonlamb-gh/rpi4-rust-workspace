use core::fmt;

pub trait Emit<W: fmt::Write> {
    fn emit(&self, out: &mut W) -> fmt::Result;
}
