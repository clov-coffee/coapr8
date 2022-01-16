use kwap_common::Array;
use kwap_msg::{Opt, OptDelta, OptNumber};

/// Something that can be stored in a CoAP Option.
///
/// These include:
/// - strings (str and String)
/// - empty (`()`)
/// - unsigned integers (`u8`, `u16`, `u32`, `u64`)
/// - bytes (anything that impls [`kwap_common::Array`])
pub trait ToOptionValue {
  /// Convert the value
  fn to_option_value<Cfg: Config>(self) -> Cfg::OptBytes;
}

impl<'a> ToOptionValue for &'a str {
  fn to_option_value<Cfg: Config>(self) -> Cfg::OptBytes {
    todo!()
  }
}

impl ToOptionValue for u16 {
  fn to_option_value<Cfg: Config>(self) -> Cfg::OptBytes {
    todo!()
  }
}

macro_rules! builder_option {
  ($rfc:literal $name:ident<$cfg:ty>(string)) => {
    paste::paste! {
      #[doc = kwap_macros::rfc_7252_doc!($rfc)]
      pub fn [<option_ $name>]<S: AsRef<str>>(mut self, number: u32, value: S) -> Self {
        self.inner.set_option(number, crate::ToOptionValue::to_option_value::<$cfg>(value.as_ref())).unwrap();
        self
      }
    }
  };
  ($rfc:literal $name:ident<$cfg:ty>($t:ty)) => {
    paste::paste! {
      #[doc = kwap_macros::rfc_7252_doc!($rfc)]
      pub fn [<option_ $name>](mut self, number: u32, value: $t) -> Self {
        self.inner.set_option(number, crate::ToOptionValue::to_option_value::<$cfg>(value)).unwrap();
        self
      }
    }
  }
}

macro_rules! common_options {
  ($cfg:ty) => {
    // crate::option::builder_option!("TODO" host<$cfg>(TODO));
    // crate::option::builder_option!("TODO" path<$cfg>(TODO));
    // crate::option::builder_option!("TODO" port<$cfg>(TODO));
    // crate::option::builder_option!("TODO" query<$cfg>(TODO));
    // crate::option::builder_option!("TODO" size1<$cfg>(TODO));
    // crate::option::builder_option!("TODO" if_match<$cfg>(TODO));
    // crate::option::builder_option!("TODO" if_none_match<$cfg>(TODO));
    // crate::option::builder_option!("TODO" proxy_scheme<$cfg>(TODO));
    // crate::option::builder_option!("TODO" proxy_uri<$cfg>(TODO));
    // crate::option::builder_option!("TODO" max_age<$cfg>(TODO));
    // crate::option::builder_option!("TODO" location_query<$cfg>(TODO));
    // crate::option::builder_option!("TODO" location_path<$cfg>(TODO));
    // crate::option::builder_option!("TODO" etag<$cfg>(TODO));
    crate::option::builder_option!("5.10.3" content_format<$cfg>(u16));
    crate::option::builder_option!("5.10.4" accept<$cfg>(u16));
  };
}

pub(crate) use builder_option;
pub(crate) use common_options;

use crate::config::Config;

pub(crate) fn add<A: Array<Item = (OptNumber, Opt<B>)>, B: Array<Item = u8>, V: IntoIterator<Item = u8>>(
  opts: &mut A,
  number: u32,
  value: V)
  -> Option<(u32, V)> {
  use kwap_msg::*;

  let exist = opts.iter_mut().find(|(OptNumber(num), _)| *num == number);

  if let Some((_, opt)) = exist {
    opt.value = OptValue(value.into_iter().collect());
    return None;
  }

  let n_opts = opts.get_size() + 1;
  let no_room = opts.max_size().map(|max| max < n_opts).unwrap_or(false);

  if no_room {
    return Some((number, value));
  }

  let num = OptNumber(number);
  let opt = Opt::<_> { delta: Default::default(),
                       value: OptValue(value.into_iter().collect()) };

  opts.extend(Some((num, opt)));

  None
}
pub(crate) fn normalize<OptNumbers: Array<Item = (OptNumber, Opt<Bytes>)>,
                  Opts: Array<Item = Opt<Bytes>>,
                  Bytes: Array<Item = u8>>(
  mut os: OptNumbers)
  -> Opts {
  if os.is_empty() {
    return Opts::default();
  }

  os.sort_by_key(|&(OptNumber(num), _)| num);
  os.into_iter().fold(Opts::default(), |mut opts, (num, mut opt)| {
                  let delta = opts.iter().fold(0u16, |n, opt| opt.delta.0 + n);
                  opt.delta = OptDelta((num.0 as u16) - delta);
                  opts.push(opt);
                  opts
                })
}

#[cfg(test)]
mod test {
  use super::*;
  use kwap_msg::OptValue;

  #[test]
  fn add_updates_when_exist() {
    let mut opts = vec![(OptNumber(0),
                         Opt::<Vec<u8>> { delta: OptDelta(0),
                                          value: OptValue(vec![]) })];

    let out = add(&mut opts, 0, vec![1]);

    assert!(out.is_none());
    assert_eq!(opts.len(), 1);
    assert_eq!(opts[0].1.value.0, vec![1]);
  }

  #[test]
  fn add_adds_when_not_exist() {
    let mut opts = Vec::<(_, Opt<Vec<u8>>)>::new();

    let out = add(&mut opts, 0, vec![1]);

    assert!(out.is_none());
    assert_eq!(opts.len(), 1);
    assert_eq!(opts[0].1.value.0, vec![1]);
  }  #[test]
  fn normalize_opts_echoes_when_empty() {
    let opts = Vec::<(OptNumber, Opt<Vec<u8>>)>::new();
    let out = normalize::<_, Vec<Opt<Vec<u8>>>, _>(opts);
    assert!(out.is_empty())
  }

  #[test]
  fn normalize_opts_works() {
    let opts: Vec<(OptNumber, Opt<Vec<u8>>)> = vec![(OptNumber(32), Default::default()),
                                                    (OptNumber(1), Default::default()),
                                                    (OptNumber(3), Default::default()),];

    let expect: Vec<Opt<Vec<u8>>> = vec![Opt { delta: OptDelta(1),
                                               ..Default::default() },
                                         Opt { delta: OptDelta(2),
                                               ..Default::default() },
                                         Opt { delta: OptDelta(29),
                                               ..Default::default() },];

    let actual = normalize::<_, Vec<Opt<Vec<u8>>>, _>(opts);

    assert_eq!(actual, expect)
  }
  #[test]
  fn add_rets_some_when_full() {
    let mut opts =
      tinyvec::ArrayVec::<[(OptNumber, Opt<Vec<u8>>); 1]>::from([(OptNumber(1), Opt::<Vec<u8>>::default())]);

    let out = add(&mut opts, 0, vec![1]);

    assert_eq!(out, Some((0, vec![1])));
  }
}
