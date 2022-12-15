use chumsky::prelude::*;
use chumsky::error::SimpleReason;
use chumsky::text::whitespace;
use rand::Rng;

use std::fmt;
use std::str::FromStr;
use std::ops::Range;

/*
ROLL: ('coin' | SYNTAX1 | SYNTAX2)
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
  pub dice_results: Vec<u32>,
  pub roll: Roll
}

impl fmt::Display for ExecutedRoll {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    if let Ok([dice_result]) = <[u32; 1]>::try_from(&self.dice_results[..]) {
      let value = dice_result as i64 + self.roll.modifier as i64;
      match self.roll.modifier {
        0 => write!(f, "**{dice_result}**")?,
        m => write!(f, "({dice_result}){} = **{value}**", DisplayModifier(m))?
      };
    } else {
      write!(f, "[")?;
      let mut values = Vec::with_capacity(self.dice_results.len());
      for (i, &dice_result) in self.dice_results.iter().enumerate() {
        if i != 0 { write!(f, ", ")? };
        let value = dice_result as i64 + self.roll.modifier as i64;
        match self.roll.modifier {
          0 => write!(f, "{dice_result}")?,
          m => write!(f, "({dice_result}){} = {value}", DisplayModifier(m))?
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
        let total = values.into_iter().sum::<i64>();
        write!(f, " {mode} = **{total}**")?;
      } else {
        let total = values.into_iter().sum::<i64>();
        write!(f, " = **{total}**")?;
      };
    };

    Ok(())
  }
}

#[derive(Debug, Clone, Copy)]
pub struct Roll {
  pub dice: u8,
  pub sides: u32,
  pub modifier: i16,
  pub mode: Option<Mode>
}

impl Roll {
  pub const COIN: Self = Roll { dice: 1, sides: 2, modifier: 0, mode: None };

  pub fn execute(self) -> ExecutedRoll {
    let mut rng = rand::thread_rng();
    let dice_results = (0..self.dice).map(|_| rng.gen_range(1..=self.sides)).collect();
    ExecutedRoll { dice_results, roll: self.normalize() }
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

type Roll1 = (((u8, u32), i16), Option<Mode>);
type Roll2 = ((u32, i16), ModeType);

impl From<Roll1> for Roll {
  fn from((((dice, sides), modifier), mode): Roll1) -> Roll {
    Roll { dice, sides, modifier, mode }
  }
}

impl From<Roll2> for Roll {
  fn from(((sides, modifier), mode_type): Roll2) -> Roll {
    let mode = Mode(mode_type, 1);
    Roll { dice: 2, sides, modifier, mode: Some(mode) }
  }
}

impl FromStr for Roll {
  type Err = ParseRollError;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    match roll().parse(s.to_ascii_lowercase()) {
      Ok(roll) => Ok(roll.normalize()),
      Err(mut err) => {
        err.truncate(8);
        Err(ParseRollError(err))
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

#[derive(Debug, Clone, Copy)]
pub enum ModeType {
  Min,
  Max
}

#[derive(Debug, Clone)]
pub struct ParseRollError(Vec<Simple<char>>);

impl fmt::Display for ParseRollError {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    for (i, err) in self.0.iter().enumerate() {
      if i != 0 { writeln!(f)? };

      if let SimpleReason::Custom(msg) = err.reason() {
        write!(f, "{msg}")?;
      } else {
        write!(f, "{err}")?;
      };

      if let Some(label) = err.label() {
        write!(f, " ({label})")?;
      };
    };

    Ok(())
  }
}

impl std::error::Error for ParseRollError {}

macro_rules! spaced {
  ($expr:expr $(,$exprn:expr)*) => {
    $expr$(.then_ignore(whitespace()).then($exprn))*
  };
}

fn or_not_with<P, T>(p: P, default: T) -> impl Parser<char, T, Error = Simple<char>> + Copy
where P: Parser<char, T, Error = Simple<char>> + Copy, T: Copy {
  p.or_not().map(move |v| v.unwrap_or(default))
}

fn roll() -> impl Parser<char, Roll, Error = Simple<char>> {
  let dice = or_not_with(dice(), 1);
  let sides = just('d').then(whitespace()).ignore_then(sides());
  let modifier = or_not_with(modifier(), 0);

  let syntax0 = just("coin").to(Roll::COIN)
    .padded().then_ignore(end()).labelled("coin flip syntax");
  let syntax1 = spaced!(dice, sides, modifier, mode1().or_not())
    .map(Roll::from).padded().then_ignore(end()).labelled("syntax 1");
  let syntax2 = spaced!(sides, modifier, mode2())
    .map(Roll::from).padded().then_ignore(end()).labelled("syntax 2");
  choice((syntax0, syntax2, syntax1))
}

fn mode1() -> impl Parser<char, Mode, Error = Simple<char>> {
  let min = choice((just("minimum"), just("min"))).to(ModeType::Min);
  let max = choice((just("maximum"), just("max"))).to(ModeType::Max);
  let i = or_not_with(int::<u8>().validate(valid_int(1)), 1);
  spaced!(choice((min, max)), i).map(Mode::from).labelled("mode: syntax 1")
}

fn mode2() -> impl Parser<char, ModeType, Error = Simple<char>> {
  let dis = choice((just("disadvantage"), just("dis"))).to(ModeType::Min);
  let adv = choice((just("advantage"), just("adv"))).to(ModeType::Max);
  choice((dis, adv)).labelled("mode: syntax 2")
}

fn dice() -> impl Parser<char, u8, Error = Simple<char>> + Copy {
  int::<u8>().validate(valid_int(1)).labelled("dice count")
}

fn sides() -> impl Parser<char, u32, Error = Simple<char>> + Copy {
  int::<u32>().validate(valid_int(2)).labelled("sides count")
}

fn modifier() -> impl Parser<char, i16, Error = Simple<char>> + Copy {
  let sign = choice((just('-'), just('+'))).then_ignore(whitespace());
  from_str(sign.chain(digits()).collect::<String>()).labelled("modifier")
}

fn int<T: FromStr>() -> impl Parser<char, T, Error = Simple<char>> + Copy
where <T as FromStr>::Err: ToString {
  from_str(digits().collect::<String>()).labelled("int")
}

fn digits() -> impl Parser<char, Vec<char>, Error = Simple<char>> + Copy {
  filter(char::is_ascii_digit).repeated().at_least(1).labelled("digits")
}

fn from_str<T: FromStr, P, S>(p: P) -> impl Parser<char, T, Error = Simple<char>> + Copy
where <T as FromStr>::Err: ToString, S: AsRef<str>, P: Parser<char, S, Error = Simple<char>> + Copy {
  p.try_map(|s, span| s.as_ref().parse::<T>().map_err(|err| Simple::custom(span, err)))
}

fn valid_int<I: Copy + Ord + fmt::Display>(min: I)
-> impl Fn(I, Range<usize>, &mut dyn FnMut(Simple<char>)) -> I + Copy {
  move |v, span, emit| {
    if v < min { emit(Simple::custom(span, format!("int may not be less than {min}"))) };
    v
  }
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
