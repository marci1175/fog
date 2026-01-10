use std::ops::Range;

use crate::error::DbgInfo;

/// This function `should` not return a None.
/// This function will return Some(DbgInfo::Default) if the combined ranges return a none (indicates an issue with indexing) until this function is stabilized.
pub fn fetch_and_merge_debug_information(
    list: &[DbgInfo],
    range: Range<usize>,
    is_ordered: bool,
) -> Option<DbgInfo>
{
    let fetched_items = list.get(range);
    // fetched_items.map(|debug_infos| combine_ranges(debug_infos, is_ordered))
    Some(
        fetched_items
            .map(|debug_infos| combine_ranges(debug_infos, is_ordered))
            .unwrap_or_default(),
    )
}

/// This function ignores whether the ranges are joint.
/// If this function with is_ordered, it will create a range based on the first and the last item of the range
/// This function will panic if an empty list is passed in
pub fn combine_ranges(debug_infos: &[DbgInfo], is_ordered: bool) -> DbgInfo
{
    if debug_infos.len() == 1 {
        return debug_infos[0];
    }

    if is_ordered {
        let start = debug_infos[0];
        let end = debug_infos[debug_infos.len() - 1];

        DbgInfo {
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

/// Compares two ranges and combines them. (Assumes theyre overlapping)
#[inline(always)]
pub fn merge_ranges(lhs: &mut DbgInfo, rhs: &DbgInfo)
{
    if lhs.char_start > rhs.char_start {
        lhs.char_start = rhs.char_start;
    }

    if lhs.char_end < rhs.char_end {
        lhs.char_end = rhs.char_end;
    }
}
