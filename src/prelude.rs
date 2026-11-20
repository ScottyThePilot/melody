#![allow(unused)]
#![allow(non_camel_case_types)]

pub use crate::utils::{Operate, OperateMut};
pub use crate::error::*;

pub use abyss::error::{Log, Print, Contextualize};
pub use rand::{Rng, RngExt, SeedableRng};
pub use itertools::Itertools;
pub use melody_random::{SeedableRngUtils, SliceRandomUtils, SliceMutRandomUtils};
pub use serenity::model::mention::Mentionable;

pub use std::collections::{HashMap, HashSet};

use std::num::NonZero;

pub type zu8 = NonZero<u8>;
pub type zu16 = NonZero<u16>;
pub type zu32 = NonZero<u32>;
pub type zu64 = NonZero<u64>;
pub type zu128 = NonZero<u128>;
pub type zusize = NonZero<usize>;
pub type zi8 = NonZero<i8>;
pub type zi16 = NonZero<i16>;
pub type zi32 = NonZero<i32>;
pub type zi64 = NonZero<i64>;
pub type zi128 = NonZero<i128>;
pub type zisize = NonZero<isize>;
