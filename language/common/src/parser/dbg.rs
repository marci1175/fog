use std::ops::Range;

use crate::error::SpanInfo;

/// This function ignores whether the ranges are joint.
/// If this function with is_ordered, it will create a range based on the first and the last item of the range
/// This function will panic if an empty list is passed in
pub fn combine_span_info(debug_infos: &[SpanInfo], is_ordered: bool) -> SpanInfo
{
    if debug_infos.len() == 1 {
        return debug_infos[0];
    }

    if is_ordered {
        let start = debug_infos[0];
        let end = debug_infos[debug_infos.len() - 1];

        SpanInfo {
            char_start: start.char_start,
            char_end: end.char_end,
        }
    }
    else {
        let mut range = debug_infos[0];

        for rhs in &debug_infos[1..] {
            merge_ranges(&mut range, rhs);
        }

        range
    }
}

/// Compares two ranges and combines them.
#[inline(always)]
pub fn merge_ranges(lhs: &mut SpanInfo, rhs: &SpanInfo)
{
    if lhs.char_start > rhs.char_start {
        lhs.char_start = rhs.char_start;
    }

    if lhs.char_end < rhs.char_end {
        lhs.char_end = rhs.char_end;
    }
}
