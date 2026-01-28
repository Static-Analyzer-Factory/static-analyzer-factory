//! `SafLogValue` trait for formatting Rust values into the SAF log DSL.
//!
//! Each type maps to a DSL value type:
//! - IDs (`EntityId` impls) → `0x` + hex
//! - Sets (`BTreeSet<T>`) → `{a,b,c}`
//! - Lists (`Vec<T>`, `&[T]`) → `[a,b,c]`
//! - Pairs (`SafPair<T>`) → `a->b`
//! - Ratios (`SafRatio`) → `n/m`
//! - Deltas (`PtsDelta<T>`) → `+{a,b}` or `-{a,b}`
//! - Primitives → bare values

use std::collections::BTreeSet;
use std::fmt::Write as _;
use std::time::Duration;

use crate::ids::EntityId;

/// Trait for formatting values into the SAF log DSL output.
pub trait SafLogValue {
    /// Append the DSL-formatted representation to `buf`.
    fn fmt_saf_log(&self, buf: &mut String);

    /// Convenience: format to a new `String`.
    fn to_saf_log(&self) -> String {
        let mut buf = String::new();
        self.fmt_saf_log(&mut buf);
        buf
    }
}

// --- Primitives ---

impl SafLogValue for bool {
    fn fmt_saf_log(&self, buf: &mut String) {
        buf.push_str(if *self { "true" } else { "false" });
    }
}

macro_rules! impl_saf_log_int {
    ($($ty:ty),*) => {
        $(
            impl SafLogValue for $ty {
                fn fmt_saf_log(&self, buf: &mut String) {
                    let _ = write!(buf, "{self}");
                }
            }
        )*
    };
}

impl_saf_log_int!(u8, u16, u32, u64, usize, i8, i16, i32, i64, isize);

impl SafLogValue for f64 {
    fn fmt_saf_log(&self, buf: &mut String) {
        let _ = write!(buf, "{self:.3}");
    }
}

// --- Strings ---

impl SafLogValue for str {
    fn fmt_saf_log(&self, buf: &mut String) {
        buf.push_str(self);
    }
}

impl SafLogValue for String {
    fn fmt_saf_log(&self, buf: &mut String) {
        buf.push_str(self);
    }
}

impl SafLogValue for &str {
    fn fmt_saf_log(&self, buf: &mut String) {
        buf.push_str(self);
    }
}

// --- IDs (u128 newtypes via EntityId) ---

impl SafLogValue for u128 {
    fn fmt_saf_log(&self, buf: &mut String) {
        let _ = write!(buf, "0x{self:032x}");
    }
}

/// Blanket impl for all `EntityId` types (`ValueId`, `LocId`, etc.).
impl<T: EntityId> SafLogValue for T {
    fn fmt_saf_log(&self, buf: &mut String) {
        self.raw().fmt_saf_log(buf);
    }
}

// --- Collections ---

impl<T: SafLogValue> SafLogValue for BTreeSet<T> {
    fn fmt_saf_log(&self, buf: &mut String) {
        buf.push('{');
        for (i, item) in self.iter().enumerate() {
            if i > 0 {
                buf.push(',');
            }
            item.fmt_saf_log(buf);
        }
        buf.push('}');
    }
}

impl<T: SafLogValue> SafLogValue for Vec<T> {
    fn fmt_saf_log(&self, buf: &mut String) {
        self.as_slice().fmt_saf_log(buf);
    }
}

impl<T: SafLogValue> SafLogValue for [T] {
    fn fmt_saf_log(&self, buf: &mut String) {
        buf.push('[');
        for (i, item) in self.iter().enumerate() {
            if i > 0 {
                buf.push(',');
            }
            item.fmt_saf_log(buf);
        }
        buf.push(']');
    }
}

// --- Duration ---

impl SafLogValue for Duration {
    fn fmt_saf_log(&self, buf: &mut String) {
        let _ = write!(buf, "{:.3}s", self.as_secs_f64());
    }
}

// --- Option ---

impl<T: SafLogValue> SafLogValue for Option<T> {
    fn fmt_saf_log(&self, buf: &mut String) {
        match self {
            Some(v) => v.fmt_saf_log(buf),
            None => buf.push_str("none"),
        }
    }
}

// --- Newtype wrappers for ambiguous types ---

/// A directed pair rendered as `a->b`.
pub struct SafPair<'a, T: SafLogValue>(pub &'a T, pub &'a T);

impl<T: SafLogValue> SafLogValue for SafPair<'_, T> {
    fn fmt_saf_log(&self, buf: &mut String) {
        self.0.fmt_saf_log(buf);
        buf.push_str("->");
        self.1.fmt_saf_log(buf);
    }
}

/// A ratio rendered as `n/m`.
pub struct SafRatio(pub usize, pub usize);

impl SafLogValue for SafRatio {
    fn fmt_saf_log(&self, buf: &mut String) {
        let _ = write!(buf, "{}/{}", self.0, self.1);
    }
}

/// A set delta rendered as `+{a,b}` (added) or `-{a,b}` (removed).
pub enum PtsDelta<'a, T: SafLogValue> {
    /// Items added.
    Added(&'a [T]),
    /// Items removed.
    Removed(&'a [T]),
}

impl<T: SafLogValue> SafLogValue for PtsDelta<'_, T> {
    fn fmt_saf_log(&self, buf: &mut String) {
        let (prefix, items) = match self {
            PtsDelta::Added(items) => ("+", *items),
            PtsDelta::Removed(items) => ("-", *items),
        };
        buf.push_str(prefix);
        buf.push('{');
        for (i, item) in items.iter().enumerate() {
            if i > 0 {
                buf.push(',');
            }
            item.fmt_saf_log(buf);
        }
        buf.push('}');
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bool() {
        assert_eq!(true.to_saf_log(), "true");
        assert_eq!(false.to_saf_log(), "false");
    }

    #[test]
    fn test_integers() {
        assert_eq!(42_u32.to_saf_log(), "42");
        assert_eq!((-1_i32).to_saf_log(), "-1");
        assert_eq!(0_usize.to_saf_log(), "0");
    }

    #[test]
    fn test_u128_id() {
        assert_eq!(
            0x1a2b_u128.to_saf_log(),
            "0x00000000000000000000000000001a2b"
        );
    }

    #[test]
    fn test_string() {
        assert_eq!("main".to_saf_log(), "main");
        assert_eq!(String::from("foo").to_saf_log(), "foo");
    }

    #[test]
    fn test_btreeset() {
        let set: BTreeSet<u32> = [3, 1, 2].into_iter().collect();
        assert_eq!(set.to_saf_log(), "{1,2,3}");
    }

    #[test]
    fn test_vec() {
        let v = vec![10_u32, 20, 30];
        assert_eq!(v.to_saf_log(), "[10,20,30]");
    }

    #[test]
    fn test_duration() {
        let d = Duration::from_secs_f64(1.2345);
        assert_eq!(d.to_saf_log(), "1.234s");
    }

    #[test]
    fn test_option() {
        let some: Option<u32> = Some(42);
        let none: Option<u32> = None;
        assert_eq!(some.to_saf_log(), "42");
        assert_eq!(none.to_saf_log(), "none");
    }

    #[test]
    fn test_pair() {
        let a = 1_u32;
        let b = 2_u32;
        assert_eq!(SafPair(&a, &b).to_saf_log(), "1->2");
    }

    #[test]
    fn test_ratio() {
        assert_eq!(SafRatio(12, 50).to_saf_log(), "12/50");
    }

    #[test]
    fn test_delta() {
        let items = [1_u32, 2, 3];
        assert_eq!(PtsDelta::Added(&items).to_saf_log(), "+{1,2,3}");
        assert_eq!(PtsDelta::Removed(&items).to_saf_log(), "-{1,2,3}");
    }
}
