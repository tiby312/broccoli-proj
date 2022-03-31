use crate::*;

use core::cmp::Ordering;

pub fn is_sorted_by<I, F>(arr: &[I], mut compare: F) -> bool
where
    F: FnMut(&I, &I) -> Option<Ordering>,
{
    arr.windows(2)
        .all(|w| compare(&w[1], &w[0]).unwrap() != Ordering::Less)
}

#[inline(always)]
pub fn compare_bots<T: Aabb>(axis: impl Axis, a: &T, b: &T) -> core::cmp::Ordering {
    let (p1, p2) = (a.get().get_range(axis).start, b.get().get_range(axis).start);
    if p1 > p2 {
        core::cmp::Ordering::Greater
    } else {
        core::cmp::Ordering::Less
    }
}

///Sorts the bots based on an axis.
#[inline(always)]
pub fn sweeper_update<I: Aabb, A: Axis>(axis: A, collision_botids: &mut [I]) {
    let sclosure = |a: &I, b: &I| -> core::cmp::Ordering { compare_bots(axis, a, b) };

    collision_botids.sort_unstable_by(sclosure);
}

pub use self::prevec::PreVec;

mod prevec {
    use crate::pmut::PMut;
    use alloc::vec::Vec;

    ///An vec api to avoid excessive dynamic allocation by reusing a Vec
    pub struct PreVec {
        vec: Vec<usize>,
    }

    impl Default for PreVec {
        fn default() -> Self {
            PreVec::new()
        }
    }

    impl PreVec {
        #[allow(dead_code)]
        #[inline(always)]
        pub fn new() -> PreVec {
            PreVec { vec: Vec::new() }
        }
        #[inline(always)]
        pub fn with_capacity(num: usize) -> PreVec {
            PreVec {
                vec: Vec::with_capacity(num),
            }
        }

        ///Take advantage of the big capacity of the original vec.
        pub fn extract_vec<'a, 'b, T>(&'a mut self) -> Vec<PMut<&'b mut T>> {
            let mut v = Vec::new();
            core::mem::swap(&mut v, &mut self.vec);
            revec::convert_empty_vec(v)
        }

        ///Return the big capacity vec
        pub fn insert_vec<T>(&mut self, vec: Vec<PMut<&'_ mut T>>) {
            let mut v = revec::convert_empty_vec(vec);
            core::mem::swap(&mut self.vec, &mut v)
        }
    }
}
