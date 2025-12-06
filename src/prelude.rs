#![allow(unused)]

pub use crate::utils::{
  Contextualize,
  Operate,
  OperateMut
};
pub use crate::{
  MelodyResult,
  MelodyError,
  MelodyFileError,
  MelodyCommandError
};

pub use defy::{Log, Print};
pub use itertools::Itertools;
pub use serenity::model::mention::Mentionable;

pub use std::collections::{HashMap, HashSet};

use std::num::NonZero;

#[allow(non_camel_case_types)] pub type zu8 = NonZero<u8>;
#[allow(non_camel_case_types)] pub type zu16 = NonZero<u16>;
#[allow(non_camel_case_types)] pub type zu32 = NonZero<u32>;
#[allow(non_camel_case_types)] pub type zu64 = NonZero<u64>;
#[allow(non_camel_case_types)] pub type zu128 = NonZero<u128>;
#[allow(non_camel_case_types)] pub type zusize = NonZero<usize>;
#[allow(non_camel_case_types)] pub type zi8 = NonZero<i8>;
#[allow(non_camel_case_types)] pub type zi16 = NonZero<i16>;
#[allow(non_camel_case_types)] pub type zi32 = NonZero<i32>;
#[allow(non_camel_case_types)] pub type zi64 = NonZero<i64>;
#[allow(non_camel_case_types)] pub type zi128 = NonZero<i128>;
#[allow(non_camel_case_types)] pub type zisize = NonZero<isize>;
