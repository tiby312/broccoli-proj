//! Contains query modules for each query algorithm.

use crate::node::*;
use crate::par;
use crate::pmut::*;
use crate::tree::build::default_axis;
use crate::tree::build::Splitter;
use crate::tree::build::SplitterEmpty;
use crate::util::*;
use alloc::vec::Vec;
use axgeom::*;
use compt::*;
use core::marker::PhantomData;

pub mod from_slice;

pub mod colfind;

pub mod draw;

pub mod knearest;

pub mod raycast;

pub mod intersect_with;

pub mod nbody;

pub mod rect;

mod tools;

///Query modules provide functions based off of this trait.
pub trait Queries<'a> {
    type T: Aabb<Num = Self::Num> + 'a;
    type Num: Num;

    /// # Examples
    ///
    ///```
    /// use broccoli::{prelude::*,bbox,rect,query::Queries};
    /// let mut bots = [bbox(rect(0,10,0,10),0)];
    /// let mut tree = broccoli::new(&mut bots);
    ///
    /// use compt::Visitor;
    /// for b in tree.vistr_mut().dfs_preorder_iter().flat_map(|n|n.into_range().iter_mut()){
    ///    *b.unpack_inner()+=1;    
    /// }
    /// assert_eq!(bots[0].inner,1);
    ///```
    #[must_use]
    fn vistr_mut(&mut self) -> VistrMut<Node<'a, Self::T>>;

    /// # Examples
    ///
    ///```
    /// use broccoli::{prelude::*,bbox,rect,query::Queries};
    /// let mut bots = [rect(0,10,0,10)];
    /// let mut tree = broccoli::new(&mut bots);
    ///
    /// use compt::Visitor;
    /// let mut test = Vec::new();
    /// for b in tree.vistr().dfs_preorder_iter().flat_map(|n|n.range.iter()){
    ///    test.push(b);
    /// }
    /// assert_eq!(test[0],&axgeom::rect(0,10,0,10));
    ///```
    #[must_use]
    fn vistr(&self) -> Vistr<Node<'a, Self::T>>;
}

///panics if a broken broccoli tree invariant is detected.
///For debugging purposes only.
pub fn assert_tree_invariants<T: Aabb>(tree: &crate::Tree<T>)
where
    T::Num: core::fmt::Debug,
{
    fn inner<A: Axis, T: Aabb>(axis: A, iter: compt::LevelIter<Vistr<Node<T>>>)
    where
        T::Num: core::fmt::Debug,
    {
        fn a_bot_has_value<N: Num>(it: impl Iterator<Item = N>, val: N) -> bool {
            for b in it {
                if b == val {
                    return true;
                }
            }
            false
        }

        let ((_depth, nn), rest) = iter.next();
        let axis_next = axis.next();

        let f = |a: &&T, b: &&T| -> Option<core::cmp::Ordering> {
            let j = a
                .get()
                .get_range(axis_next)
                .start
                .partial_cmp(&b.get().get_range(axis_next).start)
                .unwrap();
            Some(j)
        };

        {
            use is_sorted::IsSorted;
            assert!(IsSorted::is_sorted_by(&mut nn.range.iter(), f));
        }

        if let Some([start, end]) = rest {
            match nn.div {
                Some(div) => {
                    if nn.range.is_empty() {
                        assert_eq!(nn.cont.start, nn.cont.end);
                        let v: T::Num = Default::default();
                        assert_eq!(nn.cont.start, v);
                    } else {
                        let cont = nn.cont;
                        for bot in nn.range.iter() {
                            assert!(bot.get().get_range(axis).contains(div));
                        }

                        assert!(a_bot_has_value(
                            nn.range.iter().map(|b| b.get().get_range(axis).start),
                            div
                        ));

                        for bot in nn.range.iter() {
                            assert!(cont.contains_range(bot.get().get_range(axis)));
                        }

                        assert!(a_bot_has_value(
                            nn.range.iter().map(|b| b.get().get_range(axis).start),
                            cont.start
                        ));
                        assert!(a_bot_has_value(
                            nn.range.iter().map(|b| b.get().get_range(axis).end),
                            cont.end
                        ));
                    }

                    inner(axis_next, start);
                    inner(axis_next, end);
                }
                None => {
                    for (_depth, n) in start.dfs_preorder_iter().chain(end.dfs_preorder_iter()) {
                        assert!(n.range.is_empty());
                        //assert!(n.cont.is_none());
                        assert_eq!(n.cont.start, nn.cont.end);
                        let v: T::Num = Default::default();
                        assert_eq!(n.cont.start, v);

                        assert!(n.div.is_none());
                    }
                }
            }
        }
    }

    inner(default_axis(), tree.vistr().with_depth(compt::Depth(0)))
}
