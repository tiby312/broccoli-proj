//! Provides 2d broadphase collision detection.

mod oned;

//use std::borrow::Borrow;

use super::tools;
use super::*;
pub mod build;
use build::*;
pub mod handler;
use handler::*;

impl<'a, T: Aabb> Assert<'a, T> {
    ///Panics if a disconnect is detected between all colfind methods.
    pub fn assert_query(&mut self) {
        let bots = &mut self.inner;
        #[derive(PartialEq, Eq, PartialOrd, Ord, Debug)]
        pub struct CollisionPtr {
            inner: Vec<(usize, usize)>,
        }

        impl CollisionPtr {
            fn new() -> Self {
                CollisionPtr { inner: vec![] }
            }
            fn add<N>(&mut self, a: &BBox<N, usize>, b: &BBox<N, usize>) {
                let a = a.inner;
                let b = b.inner;
                let (a, b) = if a < b { (a, b) } else { (b, a) };

                self.inner.push((a, b));
            }
            pub fn finish(&mut self) {
                self.inner.sort_unstable();
            }
        }

        let mut bots: Vec<_> = bots
            .iter_mut()
            .enumerate()
            .map(|(i, x)| crate::bbox(*x.get(), i))
            .collect();
        let bots = bots.as_mut_slice();

        let naive_res = {
            let mut cc = CollisionPtr::new();
            Naive::new(bots).find_colliding_pairs(|a, b| {
                cc.add(&*a, &*b);
            });
            cc.finish();
            cc
        };

        let tree_res = {
            let mut cc = CollisionPtr::new();

            Tree::new(bots).find_colliding_pairs(|a, b| {
                cc.add(&*a, &*b);
            });
            cc.finish();
            cc
        };

        let notsort_res = {
            let mut cc = CollisionPtr::new();

            NotSortedTree::new(bots).find_colliding_pairs(|a, b| {
                cc.add(&*a, &*b);
            });
            cc.finish();
            cc
        };

        let sweep_res = {
            let mut cc = CollisionPtr::new();
            SweepAndPrune::new(bots).find_colliding_pairs(|a, b| {
                cc.add(&*a, &*b);
            });
            cc.finish();
            cc
        };

        assert_eq!(naive_res.inner.len(), sweep_res.inner.len());
        assert_eq!(naive_res.inner.len(), tree_res.inner.len());
        assert_eq!(naive_res.inner.len(), notsort_res.inner.len());

        assert_eq!(naive_res, tree_res);
        assert_eq!(naive_res, sweep_res);
        assert_eq!(naive_res, notsort_res);
    }
}

impl<'a, T: Aabb> Naive<'a, T> {
    pub fn find_colliding_pairs(&mut self, mut func: impl FnMut(AabbPin<&mut T>, AabbPin<&mut T>)) {
        queries::for_every_pair(self.inner.borrow_mut(), move |a, b| {
            if a.get().intersects_rect(b.get()) {
                func(a, b);
            }
        });
    }
}

impl<'a, T: Aabb> SweepAndPrune<'a, T> {
    pub fn find_colliding_pairs(&mut self, mut func: impl FnMut(AabbPin<&mut T>, AabbPin<&mut T>)) {
        let mut prevec = Vec::with_capacity(2048);
        let bots = AabbPin::from_mut(self.inner);
        oned::find_2d(&mut prevec, default_axis(), bots, &mut func, true);
    }

    #[cfg(feature = "parallel")]
    ///Sweep and prune algorithm.
    pub fn par_find_colliding_pairs(
        &mut self,
        mut func: impl FnMut(AabbPin<&mut T>, AabbPin<&mut T>) + Clone + Send,
    ) where
        T: Send,
    {
        let axis = default_axis();
        let mut prevec = Vec::with_capacity(2048);
        let bots = AabbPin::from_mut(self.inner);
        let a2 = axis.next();
        let _ = oned::find_par(
            &mut prevec,
            axis,
            bots,
            move |a: AabbPin<&mut T>, b: AabbPin<&mut T>| {
                if a.get().get_range(a2).intersects(b.get().get_range(a2)) {
                    func(a, b);
                }
            },
        );
    }
}

use crate::tree::splitter::Splitter;

const SEQ_FALLBACK_DEFAULT: usize = 2_400;

impl<'a, T: Aabb> NotSortedTree<'a, T> {
    pub fn find_colliding_pairs(&mut self, func: impl FnMut(AabbPin<&mut T>, AabbPin<&mut T>)) {
        self.colliding_pairs_builder(&mut NoSortNodeHandler::new(func))
            .build();
    }

    #[cfg(feature = "parallel")]
    pub fn par_find_colliding_pairs<F: FnMut(AabbPin<&mut T>, AabbPin<&mut T>)>(&mut self, func: F)
    where
        T: Send,
        T::Num: Send,
        F: Send + Clone,
    {
        self.colliding_pairs_builder(&mut NoSortNodeHandler::new(func))
            .build_par();
    }

    pub fn colliding_pairs_builder<'b, SO: NodeHandler<T>>(
        &'b mut self,
        handler: &'b mut SO,
    ) -> CollidingPairsBuilder<'a, 'b, T, SO> {
        CollidingPairsBuilder::new(self.vistr_mut(), handler)
    }
}

impl<'a, T: Aabb> Tree<'a, T> {
    pub fn find_colliding_pairs(&mut self, func: impl FnMut(AabbPin<&mut T>, AabbPin<&mut T>)) {
        self.colliding_pairs_builder(&mut DefaultNodeHandler::new(func))
            .build();
    }

    #[cfg(feature = "parallel")]
    pub fn par_find_colliding_pairs<F: FnMut(AabbPin<&mut T>, AabbPin<&mut T>)>(&mut self, func: F)
    where
        T: Send,
        T::Num: Send,
        F: Send + Clone,
    {
        self.colliding_pairs_builder(&mut DefaultNodeHandler::new(func))
            .build_par();
    }

    pub fn colliding_pairs_builder<'b, SO: NodeHandler<T>>(
        &'b mut self,
        handler: &'b mut SO,
    ) -> CollidingPairsBuilder<'a, 'b, T, SO> {
        CollidingPairsBuilder::new(self.vistr_mut(), handler)
    }
}

#[must_use]
pub struct CollidingPairsBuilder<'a, 'b, T: Aabb, SO: NodeHandler<T>> {
    vis: CollVis<'a, 'b, T>,
    pub num_seq_fallback: usize,
    pub handler: &'b mut SO,
}

impl<'a, 'b, T: Aabb, SO: NodeHandler<T>> CollidingPairsBuilder<'a, 'b, T, SO> {
    fn new(v: VistrMutPin<'b, Node<'a, T>>, handler: &'b mut SO) -> Self {
        CollidingPairsBuilder {
            vis: CollVis::new(v, default_axis().to_dyn()),
            num_seq_fallback: SEQ_FALLBACK_DEFAULT,
            handler,
        }
    }
    pub fn build(self) {
        self.vis.recurse_seq(self.handler);
    }

    #[cfg(feature = "parallel")]
    pub fn build_par(self)
    where
        T: Send,
        T::Num: Send,
        SO: Splitter + Send,
    {
        ///
        /// height_seq_fallback: if a subtree has this height, it will be processed as one unit sequentially.
        ///
        pub fn recurse_par<T: Aabb, N: NodeHandler<T>>(
            vistr: CollVis<T>,
            handler: &mut N,
            num_seq_fallback: usize,
        ) where
            T: Send,
            T::Num: Send,
            N: Splitter + Send,
        {
            if vistr.num_elem() <= num_seq_fallback {
                vistr.recurse_seq(handler);
            } else {
                let mut h2 = handler.div();
                let (n, rest) = vistr.collide_and_next(handler);
                if let Some([left, right]) = rest {
                    rayon::join(
                        || {
                            n.finish(handler);
                            recurse_par(left, handler, num_seq_fallback)
                        },
                        || recurse_par(right, &mut h2, num_seq_fallback),
                    );
                    handler.add(h2);
                } else {
                    n.finish(handler);
                }
            }
        }

        recurse_par(self.vis, self.handler, self.num_seq_fallback)
    }

    pub fn build_with_splitter<SS: Splitter>(self, splitter: &mut SS) {
        pub fn recurse_seq_splitter<T: Aabb, S: NodeHandler<T>, SS: Splitter>(
            vistr: CollVis<T>,
            splitter: &mut SS,
            func: &mut S,
        ) {
            let (n, rest) = vistr.collide_and_next(func);

            if let Some([left, right]) = rest {
                let mut s2 = splitter.div();
                n.finish(func);
                recurse_seq_splitter(left, splitter, func);
                recurse_seq_splitter(right, &mut s2, func);
                splitter.add(s2);
            } else {
                n.finish(func);
            }
        }
        recurse_seq_splitter(self.vis, splitter, self.handler)
    }
}
