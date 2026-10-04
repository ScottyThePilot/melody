use crate::common::{Error, Extra, ParserExt};

use chumsky::prelude::*;
use chumsky::text::whitespace;
use rand::{Rng, RngExt};

use std::fmt;
use std::str::FromStr;

/*
ROLL: (ROLL_COIN | ROLL_DICE)
ROLL_COIN: 'coin' (','? TARGET_COIN)?
TARGET_COIN: ('heads' | 'tails' | '1' | '2')
ROLL_DICE: (SYNTAX1 | SYNTAX2) (','? TARGET_DICE)?
TARGET_DICE: COMPARISON INT
COMPARISON: ('>=' | '>' | '<=' | '<' | '==' | '!=')
MODIFIER: ('-' | '+') INT
MODE1: ('min' | 'minimum' | 'max' | 'maximum') INT?
MODE2: ('adv' | 'advantage' | 'dis' | 'disadvantage')
SYNTAX1: INT 'd' INT MODIFIER? MODE1?
SYNTAX2: 'd' INT MODIFIER? MODE2
INT: DIGIT+
DIGIT: (ascii digits)
*/

#[derive(Debug, Clone)]
pub struct ExecutedRoll {
  pub roll_results: Vec<u32>,
  pub roll: Roll
}

impl fmt::Display for ExecutedRoll {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    let roll_results_total;

    if let Ok([roll_result]) = <[u32; 1]>::try_from(&self.roll_results[..]) {
      roll_results_total = roll_result as i64 + self.roll.modifier as i64;
      match self.roll.modifier {
        0 => write!(f, "**{roll_result}**")?,
        m => write!(f, "({roll_result}){} = **{roll_results_total}**", DisplayModifier(m))?
      };
    } else {
      write!(f, "[")?;
      let mut values = Vec::with_capacity(self.roll_results.len());
      for (i, &roll_result) in self.roll_results.iter().enumerate() {
        if i != 0 { write!(f, ", ")? };
        let value = roll_result as i64 + self.roll.modifier as i64;
        match self.roll.modifier {
          0 => write!(f, "{roll_result}")?,
          m => write!(f, "({roll_result}){} = {value}", DisplayModifier(m))?
        };

        values.push(value);
      };
      write!(f, "]")?;

      if let Some(mode @ Mode(mode_type, n)) = self.roll.mode {
        debug_assert!(n >= 1);
        match mode_type {
          ModeType::Max => values.sort_unstable_by(|a, b| Ord::cmp(b, a)),
          ModeType::Min => values.sort_unstable_by(|a, b| Ord::cmp(a, b))
        };

        let values = &values[0..(n as usize)];
        roll_results_total = values.into_iter().sum::<i64>();
        write!(f, " {mode} = **{roll_results_total}**")?;
      } else {
        roll_results_total = values.into_iter().sum::<i64>();
        write!(f, " = **{roll_results_total}**")?;
      };
    };

    if let Some(RollTarget { total, comparison }) = self.roll.target {
      let outcome = if comparison.exec(roll_results_total, total) { "SUCCESS" } else { "FAILURE" };
      write!(f, " {comparison} {total}, **{outcome}**")?;
    };

    Ok(())
  }
}

#[derive(Debug, Clone, Copy)]
pub struct RollTarget {
  pub total: i64,
  pub comparison: Comparison
}

impl RollTarget {
  pub const COIN_HEADS: RollTarget = RollTarget { total: 1, comparison: Comparison::EqualTo };
  pub const COIN_TAILS: RollTarget = RollTarget { total: 2, comparison: Comparison::EqualTo };
}

#[derive(Debug, Clone, Copy)]
pub struct Roll {
  pub dice: u8,
  pub sides: u32,
  pub modifier: i16,
  pub mode: Option<Mode>,
  pub target: Option<RollTarget>
}

impl Roll {
  pub const fn new_coin(target: Option<bool>) -> Self {
    let target = match target {
      Some(true) => Some(RollTarget::COIN_HEADS),
      Some(false) => Some(RollTarget::COIN_TAILS),
      None => None
    };

    Roll { dice: 1, sides: 2, modifier: 0, mode: None, target }
  }

  pub fn execute(self, rng: &mut (impl Rng + ?Sized)) -> ExecutedRoll {
    let roll_results = (0..self.dice).map(|_| rng.random_range(1..=self.sides)).collect();
    ExecutedRoll { roll_results, roll: self.normalize() }
  }

  fn normalize(mut self) -> Self {
    debug_assert!(self.dice >= 1);
    debug_assert!(self.sides >= 2);
    if self.dice == 1 {
      self.mode = None;
    };

    self
  }
}

impl fmt::Display for Roll {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    if self.dice > 1 { write!(f, "{}", self.dice)? };
    write!(f, "d{}{}", self.sides, DisplayModifier(self.modifier))?;
    match self.mode {
      Some(Mode(mt, 1)) if self.dice == 2 => match mt {
        ModeType::Max => write!(f, " adv")?,
        ModeType::Min => write!(f, " dis")?
      },
      Some(m) => write!(f, " {m}")?,
      None => ()
    };

    Ok(())
  }
}

type Roll1 = ((((u8, u32), i16), Option<Mode>), Option<RollTarget>);
type Roll2 = (((u32, i16), ModeType), Option<RollTarget>);

impl From<Roll1> for Roll {
  fn from(((((dice, sides), modifier), mode), target): Roll1) -> Roll {
    Roll { dice, sides, modifier, mode, target }
  }
}

impl From<Roll2> for Roll {
  fn from((((sides, modifier), mode_type), target): Roll2) -> Roll {
    let mode = Mode(mode_type, 1);
    Roll { dice: 2, sides, modifier, mode: Some(mode), target }
  }
}

impl FromStr for Roll {
  type Err = ParseRollError;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    let s = s.to_ascii_lowercase();
    match roll().parse(s.as_str()).into_result() {
      Ok(roll) => Ok(roll.normalize()),
      Err(errors) => {
        let errors = errors.into_iter().take(8)
          .map(Error::into_owned)
          .collect::<Vec<Error>>();
        Err(ParseRollError(errors))
      }
    }
  }
}

#[derive(Debug, Clone, Copy)]
pub struct Mode(pub ModeType, pub u8);

impl fmt::Display for Mode {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    f.write_str(match self.0 {
      ModeType::Max => "max",
      ModeType::Min => "min"
    })?;

    if self.1 > 1 {
      write!(f, " {}", self.1)?;
    };

    Ok(())
  }
}

impl From<(ModeType, u8)> for Mode {
  fn from((mode_type, n): (ModeType, u8)) -> Self {
    Mode(mode_type, n)
  }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ModeType {
  Min,
  Max
}

#[derive(Debug, Clone)]
pub struct ParseRollError(Vec<Error<'static>>);

impl fmt::Display for ParseRollError {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    for (i, err) in self.0.iter().enumerate() {
      if i != 0 { writeln!(f)? };

      write!(f, "{err}")?;
    };

    Ok(())
  }
}

impl std::error::Error for ParseRollError {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Comparison {
  GreaterThanEqualTo,
  GreaterThan,
  LessThanEqualTo,
  LessThan,
  EqualTo,
  NotEqualTo
}

impl Comparison {
  pub const fn exec(self, lhs: i64, rhs: i64) -> bool {
    match self {
      Self::GreaterThanEqualTo => lhs >= rhs,
      Self::GreaterThan => lhs > rhs,
      Self::LessThanEqualTo => lhs <= rhs,
      Self::LessThan => lhs < rhs,
      Self::EqualTo => lhs == rhs,
      Self::NotEqualTo => lhs != rhs
    }
  }

  pub const fn to_str(self) -> &'static str {
    match self {
      Self::GreaterThanEqualTo => "\u{2265}",
      Self::GreaterThan => ">",
      Self::LessThanEqualTo => "\u{2264}",
      Self::LessThan => "<",
      Self::EqualTo => "=",
      Self::NotEqualTo => "\u{2260}"
    }
  }
}

impl AsRef<str> for Comparison {
  fn as_ref(&self) -> &str {
    self.to_str()
  }
}

impl fmt::Display for Comparison {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.write_str(self.to_str())
  }
}

macro_rules! spaced {
  ($expr:expr $(,$exprn:expr)*) => {
    $expr$(.then_ignore(whitespace()).then($exprn))*
  };
}

fn or_not_with<'src, P, T>(p: P, default: T) -> impl Parser<'src, &'src str, T, Extra<'src>> + Copy
where P: Parser<'src, &'src str, T, Extra<'src>> + Copy, T: Copy {
  p.or_not().map(move |v| v.unwrap_or(default))
}

fn roll<'src>() -> impl Parser<'src, &'src str, Roll, Extra<'src>> {
  roll_coin().or(roll_dice()).padded().then_ignore(end())
}

fn roll_coin<'src>() -> impl Parser<'src, &'src str, Roll, Extra<'src>> {
  let target = just(',').then(whitespace()).or_not()
    .ignore_then(target_coin()).or_not();

  just("coin").then(whitespace()).ignore_then(target)
    .map(Roll::new_coin).labelled("coin syntax")
}

fn target_coin<'src>() -> impl Parser<'src, &'src str, bool, Extra<'src>> {
  choice([
    just("heads").to(true),
    just("tails").to(false),
    just("1").to(true),
    just("2").to(false)
  ])
}

fn roll_dice<'src>() -> impl Parser<'src, &'src str, Roll, Extra<'src>> {
  let dice = or_not_with(dice(), 1);
  let sides = just('d').then(whitespace()).ignore_then(sides());
  let modifier = or_not_with(modifier(), 0);
  let target = just(',').then(whitespace()).or_not()
    .ignore_then(target_dice()).or_not()
    .boxed();

  let syntax1 = spaced!(dice, sides, modifier, mode1().or_not(), target.clone())
    .map(Roll::from).labelled("dice syntax 1");
  let syntax2 = spaced!(sides, modifier, mode2(), target)
    .map(Roll::from).labelled("dice syntax 2");
  choice((syntax2, syntax1))
}

fn target_dice<'src>() -> impl Parser<'src, &'src str, RollTarget, Extra<'src>> {
  spaced!(comparison(), int::<i64>()).map(|(comparison, total)| RollTarget { total, comparison })
}

fn comparison<'src>() -> impl Parser<'src, &'src str, Comparison, Extra<'src>> {
  choice([
    just("\u{2265}").to(Comparison::GreaterThanEqualTo),
    just(">=").to(Comparison::GreaterThanEqualTo),
    just(">").to(Comparison::GreaterThan),
    just("\u{2264}").to(Comparison::LessThanEqualTo),
    just("<=").to(Comparison::LessThanEqualTo),
    just("<").to(Comparison::LessThan),
    just("\u{2260}").to(Comparison::NotEqualTo),
    just("!=").to(Comparison::NotEqualTo),
    just("==").to(Comparison::EqualTo),
    just("=").to(Comparison::EqualTo),
  ])
}

fn mode1<'src>() -> impl Parser<'src, &'src str, Mode, Extra<'src>> {
  let min = choice((just("minimum"), just("min"))).to(ModeType::Min);
  let max = choice((just("maximum"), just("max"))).to(ModeType::Max);
  let i = or_not_with(int::<u8>().validate_simple_range(1..), 1);
  spaced!(choice((min, max)), i).map(Mode::from).labelled("mode: syntax 1")
}

fn mode2<'src>() -> impl Parser<'src, &'src str, ModeType, Extra<'src>> {
  let dis = choice((just("disadvantage"), just("dis"))).to(ModeType::Min);
  let adv = choice((just("advantage"), just("adv"))).to(ModeType::Max);
  choice((dis, adv)).labelled("mode: syntax 2")
}

fn dice<'src>() -> impl Parser<'src, &'src str, u8, Extra<'src>> + Copy {
  int::<u8>().validate_simple_range(1..).labelled("dice count")
}

fn sides<'src>() -> impl Parser<'src, &'src str, u32, Extra<'src>> + Copy {
  int::<u32>().validate_simple_range(2..).labelled("sides count")
}

fn modifier<'src>() -> impl Parser<'src, &'src str, i16, Extra<'src>> + Copy {
  let sign = choice((just('-'), just('+'))).then_ignore(whitespace());
  sign.then(digits()).to_slice().try_from_str::<i16>().labelled("modifier")
}

fn int<'src, T: FromStr>() -> impl Parser<'src, &'src str, T, Extra<'src>> + Copy
where <T as FromStr>::Err: ToString {
  digits().try_from_str::<T>().labelled("int")
}

fn digits<'src>() -> impl Parser<'src, &'src str, &'src str, Extra<'src>> + Copy {
  one_of("0123456789").repeated().at_least(1).to_slice().labelled("digits")
}



#[derive(Debug, Clone, Copy)]
struct DisplayModifier(i16);

impl fmt::Display for DisplayModifier {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    let modifier = (self.0 as i32).abs() as u16;
    if self.0.is_positive() {
      write!(f, " + {modifier}")
    } else {
      write!(f, " - {modifier}")
    }
  }
}
