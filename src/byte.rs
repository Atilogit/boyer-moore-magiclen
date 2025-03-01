use alloc::{
    fmt::{self, Debug, Formatter},
    string::String,
    vec::Vec,
};
use core::ops::Deref;

// TODO Searchable

#[allow(clippy::len_without_is_empty)]
pub trait BMByteSearchable {
    type Iter<'it>: Iterator<Item = u8>
    where
        Self: 'it;

    fn len(&self) -> usize;

    fn value_at(&self, index: usize) -> u8;

    fn iter(&self) -> Self::Iter<'_>;
}

impl BMByteSearchable for String {
    type Iter<'it> = core::iter::Copied<core::slice::Iter<'it, u8>>;

    #[inline]
    fn len(&self) -> usize {
        String::len(self)
    }

    #[inline]
    fn value_at(&self, index: usize) -> u8 {
        self.as_bytes()[index]
    }

    #[inline]
    fn iter(&self) -> Self::Iter<'_> {
        self.as_bytes().iter().copied()
    }
}

impl BMByteSearchable for &str {
    type Iter<'it> = core::iter::Copied<core::slice::Iter<'it, u8>> where Self: 'it;

    #[inline]
    fn len(&self) -> usize {
        str::len(self)
    }

    #[inline]
    fn value_at(&self, index: usize) -> u8 {
        unsafe { (*(*self as *const str as *const [u8]))[index] }
    }

    #[inline]
    fn iter(&self) -> Self::Iter<'_> {
        self.as_bytes().iter().copied()
    }
}

impl BMByteSearchable for dyn Deref<Target = [u8]> {
    type Iter<'it> = core::iter::Copied<core::slice::Iter<'it, u8>>;

    #[inline]
    fn len(&self) -> usize {
        <[u8]>::len(self)
    }

    #[inline]
    fn value_at(&self, index: usize) -> u8 {
        self[index]
    }

    #[inline]
    fn iter(&self) -> Self::Iter<'_> {
        <[u8]>::iter(self).copied()
    }
}

impl BMByteSearchable for Vec<u8> {
    type Iter<'it> = core::iter::Copied<core::slice::Iter<'it, u8>>;

    #[inline]
    fn len(&self) -> usize {
        Self::len(self)
    }

    #[inline]
    fn value_at(&self, index: usize) -> u8 {
        self[index]
    }

    #[inline]
    fn iter(&self) -> Self::Iter<'_> {
        self.as_slice().iter().copied()
    }
}

impl<T: BMByteSearchable> BMByteSearchable for &T {
    type Iter<'it> = T::Iter<'it> where Self: 'it;

    #[inline]
    fn len(&self) -> usize {
        T::len(self)
    }

    #[inline]
    fn value_at(&self, index: usize) -> u8 {
        T::value_at(self, index)
    }

    #[inline]
    fn iter(&self) -> Self::Iter<'_> {
        T::iter(self)
    }
}

// TODO BasCharShiftMap

pub struct BMByteBadCharShiftMap {
    t: [usize; 256],
}

impl Debug for BMByteBadCharShiftMap {
    #[inline]
    fn fmt(&self, f: &mut Formatter) -> Result<(), fmt::Error> {
        debug_helper::impl_debug_for_struct!(BMByteBadCharShiftMap, f, self, let .t = self.t.as_ref());
    }
}

impl Deref for BMByteBadCharShiftMap {
    type Target = [usize];

    #[inline]
    fn deref(&self) -> &[usize] {
        self.t.as_ref()
    }
}

pub struct BMByteBadCharShiftMapRev {
    t: [usize; 256],
}

impl Debug for BMByteBadCharShiftMapRev {
    #[inline]
    fn fmt(&self, f: &mut Formatter) -> Result<(), fmt::Error> {
        debug_helper::impl_debug_for_struct!(BMByteBadCharShiftMapRev, f, self, let .t = self.t.as_ref());
    }
}

impl Deref for BMByteBadCharShiftMapRev {
    type Target = [usize];

    #[inline]
    fn deref(&self) -> &[usize] {
        self.t.as_ref()
    }
}

impl BMByteBadCharShiftMap {
    pub fn create_bad_char_shift_map<T: BMByteSearchable>(
        pattern: T,
    ) -> Option<BMByteBadCharShiftMap> {
        let pattern_len = pattern.len();

        if pattern_len == 0 {
            return None;
        }

        let pattern_len_dec = pattern_len - 1;

        let mut bad_char_shift_map = [pattern_len; 256];

        for (i, c) in pattern.iter().take(pattern_len_dec).map(|c| c as usize).enumerate() {
            bad_char_shift_map[c] = pattern_len_dec - i;
        }

        Some(BMByteBadCharShiftMap { t: bad_char_shift_map })
    }
}

impl BMByteBadCharShiftMapRev {
    pub fn create_bad_char_shift_map<'pat, T: BMByteSearchable>(
        pattern: &'pat T,
    ) -> Option<BMByteBadCharShiftMapRev>
    where
        T::Iter<'pat>: ExactSizeIterator + DoubleEndedIterator,
    {
        let pattern_len = pattern.len();

        if pattern_len == 0 {
            return None;
        }

        let pattern_len_dec = pattern_len - 1;

        let mut bad_char_shift_map = [pattern_len; 256];

        for (i, c) in
            pattern.iter().enumerate().rev().take(pattern_len_dec).map(|(i, c)| (i, c as usize))
        {
            bad_char_shift_map[c] = i;
        }

        Some(BMByteBadCharShiftMapRev { t: bad_char_shift_map })
    }
}

// TODO BM

/// Using Boyer-Moore-MagicLen to search byte sub-sequences in any byte sequence, including self-synchronizing string encoding data such as UTF-8.
#[derive(Debug)]
pub struct BMByte {
    bad_char_shift_map: BMByteBadCharShiftMap,
    bad_char_shift_map_rev: BMByteBadCharShiftMapRev,
    pattern: Vec<u8>,
}

impl BMByte {
    /// Create a `BMByte` instance from a pattern (the needle).
    ///
    /// ```
    /// use boyer_moore_magiclen::BMByte;
    ///
    /// let bmb = BMByte::from(&"oocoo").unwrap();
    /// ```
    pub fn from<'pat, T: BMByteSearchable + 'pat>(pattern: &'pat T) -> Option<BMByte>
    where
        T::Iter<'pat>: ExactSizeIterator + DoubleEndedIterator,
    {
        let bad_char_shift_map = BMByteBadCharShiftMap::create_bad_char_shift_map(&pattern)?;
        let bad_char_shift_map_rev = BMByteBadCharShiftMapRev::create_bad_char_shift_map(pattern)?;

        Some(BMByte {
            bad_char_shift_map,
            bad_char_shift_map_rev,
            pattern: pattern.iter().collect(),
        })
    }
}

// TODO Find Full

impl BMByte {
    /// Find and return the positions of all matched sub-sequences in any text (the haystack).
    ///
    /// ```
    /// use boyer_moore_magiclen::BMByte;
    ///
    /// let bmb = BMByte::from(&"oocoo").unwrap();
    ///
    /// assert_eq!(vec![1, 4, 7], bmb.find_full_all_in("coocoocoocoo"));
    /// ```
    pub fn find_full_all_in<T: BMByteSearchable>(&self, text: T) -> Vec<usize> {
        find_full(text, &self.pattern, &self.bad_char_shift_map, 0)
    }

    /// Find and return the positions of matched sub-sequences in any text (the haystack). If the `limit` is set to `0`, all sub-sequences will be found.
    ///
    /// ```
    /// use boyer_moore_magiclen::BMByte;
    ///
    /// let bmb = BMByte::from(&"oocoo").unwrap();
    ///
    /// assert_eq!(vec![1, 4], bmb.find_full_in("coocoocoocoo", 2));
    /// ```
    pub fn find_full_in<T: BMByteSearchable>(&self, text: T, limit: usize) -> Vec<usize> {
        find_full(text, &self.pattern, &self.bad_char_shift_map, limit)
    }
}

impl BMByte {
    /// Find and return the positions of all matched sub-sequences in any text (the haystack) from its tail to its head.
    ///
    /// ```
    /// use boyer_moore_magiclen::BMByte;
    ///
    /// let bmb = BMByte::from(&"oocoo").unwrap();
    ///
    /// assert_eq!(vec![7, 4, 1], bmb.rfind_full_all_in("coocoocoocoo"));
    /// ```
    pub fn rfind_full_all_in<T: BMByteSearchable>(&self, text: T) -> Vec<usize> {
        rfind_full(text, &self.pattern, &self.bad_char_shift_map_rev, 0)
    }

    /// Find and return the positions of matched sub-sequences in any text (the haystack) from its tail to its head. If the `limit` is set to `0`, all sub-sequences will be found.
    ///
    /// ```
    /// use boyer_moore_magiclen::BMByte;
    ///
    /// let bmb = BMByte::from(&"oocoo").unwrap();
    ///
    /// assert_eq!(vec![7, 4], bmb.rfind_full_in("coocoocoocoo", 2));
    /// ```
    pub fn rfind_full_in<T: BMByteSearchable>(&self, text: T, limit: usize) -> Vec<usize> {
        rfind_full(text, &self.pattern, &self.bad_char_shift_map_rev, limit)
    }
}

pub fn find_full<'pat, TT: BMByteSearchable, TP: BMByteSearchable>(
    text: TT,
    pattern: &'pat TP,
    bad_char_shift_map: &BMByteBadCharShiftMap,
    limit: usize,
) -> Vec<usize>
where
    TP::Iter<'pat>: ExactSizeIterator + DoubleEndedIterator,
{
    let text_len = text.len();
    let pattern_len = pattern.len();

    if text_len == 0 || pattern_len == 0 || text_len < pattern_len {
        return vec![];
    }

    let pattern_len_dec = pattern_len - 1;

    let last_pattern_char = pattern.value_at(pattern_len_dec);

    let mut shift = 0;

    let end_index = text_len - pattern_len;

    let mut result = vec![];

    'outer: loop {
        for (i, pc) in pattern.iter().enumerate().rev() {
            if text.value_at(shift + i) != pc {
                let p = shift + pattern_len;
                if p == text_len {
                    break 'outer;
                }
                shift += bad_char_shift_map[text.value_at(shift + pattern_len_dec) as usize].max({
                    let c = text.value_at(p);

                    if c == last_pattern_char {
                        1
                    } else {
                        bad_char_shift_map[c as usize] + 1
                    }
                });
                if shift > end_index {
                    break 'outer;
                }
                continue 'outer;
            }
        }
        result.push(shift);

        if shift == end_index {
            break;
        }

        if result.len() == limit {
            break;
        }

        shift += bad_char_shift_map[text.value_at(shift + pattern_len_dec) as usize].max({
            let c = text.value_at(shift + pattern_len);

            if c == last_pattern_char {
                1
            } else {
                bad_char_shift_map[c as usize] + 1
            }
        });
        if shift > end_index {
            break;
        }
    }

    result
}

pub fn rfind_full<TT: BMByteSearchable, TP: BMByteSearchable>(
    text: TT,
    pattern: TP,
    bad_char_shift_map: &BMByteBadCharShiftMapRev,
    limit: usize,
) -> Vec<usize> {
    let text_len = text.len();
    let pattern_len = pattern.len();

    if text_len == 0 || pattern_len == 0 || text_len < pattern_len {
        return vec![];
    }

    let pattern_len_dec = pattern_len - 1;

    let first_pattern_char = pattern.value_at(0);

    let mut shift = text_len - 1;

    let start_index = pattern_len_dec;

    let mut result = vec![];

    'outer: loop {
        for (i, pc) in pattern.iter().enumerate() {
            if text.value_at(shift - pattern_len_dec + i) != pc {
                if shift < pattern_len {
                    break 'outer;
                }
                let s = bad_char_shift_map[text.value_at(shift - pattern_len_dec) as usize].max({
                    let c = text.value_at(shift - pattern_len);

                    if c == first_pattern_char {
                        1
                    } else {
                        bad_char_shift_map[c as usize] + 1
                    }
                });
                if shift < s {
                    break 'outer;
                }
                shift -= s;
                if shift < start_index {
                    break 'outer;
                }
                continue 'outer;
            }
        }
        result.push(shift - pattern_len_dec);

        if shift == start_index {
            break;
        }

        if result.len() == limit {
            break;
        }

        let s = bad_char_shift_map[text.value_at(shift - pattern_len_dec) as usize].max({
            let c = text.value_at(shift - pattern_len);

            if c == first_pattern_char {
                1
            } else {
                bad_char_shift_map[c as usize] + 1
            }
        });
        if shift < s {
            break;
        }
        shift -= s;
        if shift < start_index {
            break;
        }
    }

    result
}

// TODO Find

impl BMByte {
    /// Find and return the positions of all matched sub-sequences in any text (the haystack) but not including the overlap.
    ///
    /// ```
    /// use boyer_moore_magiclen::BMByte;
    ///
    /// let bmb = BMByte::from(&"oocoo").unwrap();
    ///
    /// assert_eq!(vec![1, 7], bmb.find_all_in("coocoocoocoo"));
    /// ```
    pub fn find_all_in<T: BMByteSearchable>(&self, text: T) -> Vec<usize> {
        find(text, &self.pattern, &self.bad_char_shift_map, 0)
    }

    /// Find and return the position of the first matched sub-sequence in any text (the haystack).
    ///
    /// ```
    /// use boyer_moore_magiclen::BMByte;
    ///
    /// let bmb = BMByte::from(&"oocoo").unwrap();
    ///
    /// assert_eq!(Some(1), bmb.find_first_in("coocoocoocoo"));
    /// ```
    pub fn find_first_in<T: BMByteSearchable>(&self, text: T) -> Option<usize> {
        find(text, &self.pattern, &self.bad_char_shift_map, 1).first().copied()
    }

    /// Find and return the positions of matched sub-sequences in any text (the haystack) but not including the overlap. If the `limit` is set to `0`, all sub-sequences will be found.
    ///
    /// ```
    /// use boyer_moore_magiclen::BMByte;
    ///
    /// let bmb = BMByte::from(&"oocoo").unwrap();
    ///
    /// assert_eq!(vec![1], bmb.find_in("coocoocoocoo", 1));
    /// ```
    pub fn find_in<T: BMByteSearchable>(&self, text: T, limit: usize) -> Vec<usize> {
        find(text, &self.pattern, &self.bad_char_shift_map, limit)
    }
}

impl BMByte {
    /// Find and return the positions of all matched sub-sequences in any text (the haystack) but not including the overlap from its tail to its head.
    ///
    /// ```
    /// use boyer_moore_magiclen::BMByte;
    ///
    /// let bmb = BMByte::from(&"oocoo").unwrap();
    ///
    /// assert_eq!(vec![7, 1], bmb.rfind_all_in("coocoocoocoo"));
    /// ```
    pub fn rfind_all_in<T: BMByteSearchable>(&self, text: T) -> Vec<usize> {
        rfind(text, &self.pattern, &self.bad_char_shift_map_rev, 0)
    }

    /// Find and return the position of the first matched sub-sequence in any text (the haystack) from its tail to its head.
    ///
    /// ```
    /// use boyer_moore_magiclen::BMByte;
    ///
    /// let bmb = BMByte::from(&"oocoo").unwrap();
    ///
    /// assert_eq!(Some(7), bmb.rfind_first_in("coocoocoocoo"));
    /// ```
    pub fn rfind_first_in<T: BMByteSearchable>(&self, text: T) -> Option<usize> {
        rfind(text, &self.pattern, &self.bad_char_shift_map_rev, 1).first().copied()
    }

    /// Find and return the positions of matched sub-sequences in any text (the haystack) but not including the overlap from its tail to its head. If the `limit` is set to `0`, all sub-sequences will be found.
    ///
    /// ```
    /// use boyer_moore_magiclen::BMByte;
    ///
    /// let bmb = BMByte::from(&"oocoo").unwrap();
    ///
    /// assert_eq!(vec![7], bmb.rfind_in("coocoocoocoo", 1));
    /// ```
    pub fn rfind_in<T: BMByteSearchable>(&self, text: T, limit: usize) -> Vec<usize> {
        rfind(text, &self.pattern, &self.bad_char_shift_map_rev, limit)
    }
}

pub fn find<'pat, TT: BMByteSearchable, TP: BMByteSearchable>(
    text: TT,
    pattern: &'pat TP,
    bad_char_shift_map: &BMByteBadCharShiftMap,
    limit: usize,
) -> Vec<usize>
where
    TP::Iter<'pat>: ExactSizeIterator + DoubleEndedIterator,
{
    let text_len = text.len();
    let pattern_len = pattern.len();

    if text_len == 0 || pattern_len == 0 || text_len < pattern_len {
        return vec![];
    }

    let pattern_len_dec = pattern_len - 1;

    let last_pattern_char = pattern.value_at(pattern_len_dec);

    let mut shift = 0;

    let end_index = text_len - pattern_len;

    let mut result = vec![];

    'outer: loop {
        for (i, pc) in pattern.iter().enumerate().rev() {
            if text.value_at(shift + i) != pc {
                let p = shift + pattern_len;
                if p == text_len {
                    break 'outer;
                }
                shift += bad_char_shift_map[text.value_at(shift + pattern_len_dec) as usize].max({
                    let c = text.value_at(p);

                    if c == last_pattern_char {
                        1
                    } else {
                        bad_char_shift_map[c as usize] + 1
                    }
                });
                if shift > end_index {
                    break 'outer;
                }
                continue 'outer;
            }
        }
        result.push(shift);

        if shift == end_index {
            break;
        }

        if result.len() == limit {
            break;
        }

        shift += pattern_len;
        if shift > end_index {
            break;
        }
    }

    result
}

pub fn rfind<TT: BMByteSearchable, TP: BMByteSearchable>(
    text: TT,
    pattern: TP,
    bad_char_shift_map: &BMByteBadCharShiftMapRev,
    limit: usize,
) -> Vec<usize> {
    let text_len = text.len();
    let pattern_len = pattern.len();

    if text_len == 0 || pattern_len == 0 || text_len < pattern_len {
        return vec![];
    }

    let pattern_len_dec = pattern_len - 1;

    let first_pattern_char = pattern.value_at(0);

    let mut shift = text_len - 1;

    let start_index = pattern_len_dec;

    let mut result = vec![];

    'outer: loop {
        for (i, pc) in pattern.iter().enumerate() {
            if text.value_at(shift - pattern_len_dec + i) != pc {
                if shift < pattern_len {
                    break 'outer;
                }
                let s = bad_char_shift_map[text.value_at(shift - pattern_len_dec) as usize].max({
                    let c = text.value_at(shift - pattern_len);

                    if c == first_pattern_char {
                        1
                    } else {
                        bad_char_shift_map[c as usize] + 1
                    }
                });
                if shift < s {
                    break 'outer;
                }
                shift -= s;
                if shift < start_index {
                    break 'outer;
                }
                continue 'outer;
            }
        }
        result.push(shift - pattern_len_dec);

        if shift == start_index {
            break;
        }

        if result.len() == limit {
            break;
        }

        shift -= pattern_len;
        if shift < start_index {
            break;
        }
    }

    result
}
