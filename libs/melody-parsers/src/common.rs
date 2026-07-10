use chumsky::prelude::*;
use chumsky::extra::ParserExtra;

use std::str::FromStr;
use std::ops::{Bound, RangeBounds};
use std::fmt;



pub type Error<'src> = chumsky::error::Rich<'src, char>;
pub type Extra<'src> = chumsky::extra::Err<Error<'src>>;



pub trait ParserExt<'src, I, O, E>: Parser<'src, I, O, E>
where
  I: Input<'src, Span = SimpleSpan>,
  E: ParserExtra<'src, I, Error = Rich<'src, I::Token>>
{
  fn validate_simple_range<R>(self, range: R) -> chumsky::combinator::Validate<Self, O, impl Fn(
    O, &mut chumsky::input::MapExtra<'src, '_, I, E>, &mut chumsky::input::Emitter<E::Error>
  ) -> O + Copy>
  where R: RangeBounds<O>, O: Copy + PartialOrd + fmt::Display, Self: Sized {
    let start_bound = range.start_bound().cloned();
    let end_bound = range.end_bound().cloned();
    self.validate_simple(move |value| -> Result<(), ValidateRangeError<O>> {
      if !(start_bound, end_bound).contains(value) {
        Err(ValidateRangeError { start_bound, end_bound })
      } else {
        Ok(())
      }
    })
  }

  fn validate_simple<F, OE>(self, f: F) -> chumsky::combinator::Validate<Self, O, impl Fn(
    O, &mut chumsky::input::MapExtra<'src, '_, I, E>, &mut chumsky::input::Emitter<E::Error>
  ) -> O + Copy>
  where F: Fn(&O) -> Result<(), OE> + Copy, OE: ToString, Self: Sized {
    self.validate(move |value, map_extra, emitter| -> O {
      if let Err(err) = f(&value) {
        emitter.emit(Rich::custom(map_extra.span(), err));
      };

      value
    })
  }

  fn try_from_str<U: FromStr>(self) -> chumsky::combinator::TryMap<Self, O, fn(O, I::Span) -> Result<U, E::Error>>
  where O: AsRef<str>, U: FromStr, <U as FromStr>::Err: ToString, Self: Sized {
    self.try_map(|value: O, span: I::Span| -> Result<U, E::Error> {
      value.as_ref().parse::<U>().map_err(|err| Rich::custom(span, err))
    })
  }
}

impl<'src, P, I, O, E> ParserExt<'src, I, O, E> for P
where
  P: Parser<'src, I, O, E>,
  I: Input<'src, Span = SimpleSpan>,
  E: ParserExtra<'src, I, Error = Rich<'src, I::Token>>
{}



#[derive(Debug, Clone, Copy)]
struct ValidateRangeError<T> {
  start_bound: Bound<T>,
  end_bound: Bound<T>
}

impl<T> fmt::Display for ValidateRangeError<T>
where T: fmt::Display {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    let start = match &self.start_bound {
      Bound::Included(start) => Some(("less than or equal to", start)),
      Bound::Excluded(start) => Some(("less than", start)),
      Bound::Unbounded => None
    };

    let end = match &self.end_bound {
      Bound::Included(end) => Some(("greater than or equal to", end)),
      Bound::Excluded(end) => Some(("greater than", end)),
      Bound::Unbounded => None
    };

    match (start, end) {
      (Some((start_message, start_bound)), Some((end_message, end_bound))) => {
        write!(f, "value must be {start_message} {start_bound} and {end_message} {end_bound}")
      },
      (Some((either_message, either_bound)), None) | (None, Some((either_message, either_bound))) => {
        write!(f, "value must be {either_message} {either_bound}")
      },
      (None, None) => {
        write!(f, "value may be anything")
      }
    }
  }
}
