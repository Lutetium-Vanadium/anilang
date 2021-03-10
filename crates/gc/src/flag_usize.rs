use std::cell::Cell;
use std::fmt;
use std::mem;

const USIZE_BIT_WIDTH: usize = mem::size_of::<usize>() * 8;

const FLAG_BIT_OFFSET: usize = USIZE_BIT_WIDTH - 1;
const FLAG_MASK: usize = 1 << FLAG_BIT_OFFSET;

const USIZE_MASK: usize = !FLAG_MASK;

/// A usize and a bool within the size of usize.
///
/// The usize must not exceed `usize::MAX / 2`. Exceeding it will cause a panic.
pub(crate) struct FlagUsize {
    flags: Cell<usize>,
}

impl FlagUsize {
    /// Creates a `FlagUsize`.
    ///
    /// # Panics
    /// if n exceeds `usize::MAX / 2`, it will panic.
    pub fn new(n: usize, flag: bool) -> Self {
        assert!(
            n <= USIZE_MASK,
            "FlagUsize: {} greater than max {}",
            n,
            USIZE_MASK
        );

        FlagUsize {
            flags: Cell::new(n | (flag as usize) << FLAG_BIT_OFFSET),
        }
    }

    /// Gets the flag.
    pub fn get_flag(&self) -> bool {
        (self.flags.get() & FLAG_MASK) != 0
    }

    /// Sets the flag.
    pub fn set_flag(&self, flag: bool) {
        self.flags
            .set(self.get_usize() | (flag as usize) << FLAG_BIT_OFFSET);
    }

    /// Gets the usize.
    pub fn get_usize(&self) -> usize {
        self.flags.get() & USIZE_MASK
    }

    /// Sets the usize.
    ///
    /// # Panics
    /// if n exceeds `usize::MAX / 2`, it will panic.
    pub fn set_usize(&self, n: usize) {
        assert!(
            n <= USIZE_MASK,
            "FlagUsize: {} greater than max {}",
            n,
            USIZE_MASK
        );

        self.flags.set(n | (self.flags.get() & FLAG_MASK));
    }
}

impl fmt::Debug for FlagUsize {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("FlagUsize")
            .field("n", &self.get_usize())
            .field("flag", &self.get_flag())
            .finish()
    }
}

impl fmt::Binary for FlagUsize {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if f.alternate() {
            write!(f, "{:#0w$b}", self.flags.get(), w = USIZE_BIT_WIDTH)
        } else {
            write!(
                f,
                "{} {:0w$b}",
                self.get_flag() as u8,
                self.get_usize(),
                w = FLAG_BIT_OFFSET
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_state(flags: &FlagUsize, n: usize, flag: bool) {
        let mut state = n;
        if flag {
            state |= FLAG_MASK;
        }

        assert_eq!(flags.get_usize(), n);
        assert_eq!(flags.get_flag(), flag);
        assert_eq!(flags.flags.get(), state);
    }

    #[test]
    fn test_flag_usize() {
        let flags = &FlagUsize::new(1, false);

        test_state(flags, 1, false);

        flags.set_flag(true);
        test_state(flags, 1, true);

        flags.set_usize(2);
        test_state(flags, 2, true);

        flags.set_flag(false);
        test_state(flags, 2, false);

        flags.set_usize(0);
        test_state(flags, 0, false);
    }

    fn set_above_max() {
        let flags = &FlagUsize::new(10, true);

        test_state(flags, 10, true);

        flags.set_usize((usize::MAX >> 1) + 1);
    }

    #[cfg(target_pointer_width = "64")]
    #[test]
    #[should_panic(
        expected = "FlagUsize: 9223372036854775808 greater than max 9223372036854775807"
    )]
    fn test_set_above_max() {
        set_above_max();
    }

    #[cfg(target_pointer_width = "32")]
    #[test]
    #[should_panic(expected = "FlagUsize: 2147483648 greater than max 2147483647")]
    fn test_set_above_max() {
        set_above_max();
    }
}
