//!
//! Tree building blocks
//!

use super::*;

#[must_use]
pub struct NodeFinisher<'a, T: Aabb, S> {
    axis: AxisDyn,
    div: Option<T::Num>, //This can be null if there are no bots left at all
    mid: &'a mut [T],
    sorter: S,
    num_elem: usize,
}
impl<'a, T: Aabb, S: Sorter<T>> NodeFinisher<'a, T, S> {
    pub fn get_num_elem(&self) -> usize {
        self.num_elem
    }
    #[inline(always)]
    #[must_use]
    pub fn finish(self) -> Node<'a, T> {
        fn create_cont<A: Axis, T: Aabb>(axis: A, middle: &[T]) -> axgeom::Range<T::Num> {
            match middle.split_first() {
                Some((first, rest)) => {
                    let mut min = first.get().get_range(axis).start;
                    let mut max = first.get().get_range(axis).end;

                    for a in rest.iter() {
                        let start = &a.get().get_range(axis).start;
                        let end = &a.get().get_range(axis).end;

                        if *start < min {
                            min = *start;
                        }

                        if *end > max {
                            max = *end;
                        }
                    }

                    axgeom::Range {
                        start: min,
                        end: max,
                    }
                }
                None => axgeom::Range {
                    start: Default::default(),
                    end: Default::default(),
                },
            }
        }

        let cont = match self.axis {
            AxisDyn::X => {
                self.sorter.sort(axgeom::XAXIS.next(), self.mid);
                create_cont(axgeom::XAXIS, self.mid)
            }
            AxisDyn::Y => {
                self.sorter.sort(axgeom::YAXIS.next(), self.mid);
                create_cont(axgeom::YAXIS, self.mid)
            }
        };

        Node {
            num_elem: self.num_elem,
            range: AabbPin::new(self.mid),
            cont,
            div: self.div,
        }
    }
}

///
/// The main primitive to build a tree.
///
pub struct TreeBuildVisitor<'a, T, S> {
    bots: &'a mut [T],
    current_height: usize,
    sorter: S,
    axis: AxisDyn,
}

pub struct NodeBuildResult<'a, T: Aabb, S> {
    pub node: NodeFinisher<'a, T, S>,
    pub rest: Option<[TreeBuildVisitor<'a, T, S>; 2]>,
}

impl<'a, T: Aabb, S: Sorter<T>> TreeBuildVisitor<'a, T, S> {
    pub fn get_bots(&self) -> &[T] {
        self.bots
    }
    #[must_use]
    pub fn new(num_levels: usize, bots: &'a mut [T], sorter: S) -> TreeBuildVisitor<'a, T, S> {
        assert!(num_levels >= 1);
        TreeBuildVisitor {
            bots,
            current_height: num_levels - 1,
            sorter,
            axis: default_axis().to_dyn(),
        }
    }
    #[must_use]
    pub fn get_height(&self) -> usize {
        self.current_height
    }
    #[must_use]
    pub fn build_and_next(self) -> NodeBuildResult<'a, T, S> {
        //leaf case
        if self.current_height == 0 {
            let node = NodeFinisher {
                mid: self.bots,
                div: None,
                axis: self.axis,
                sorter: self.sorter,
                num_elem: 0,
            };

            NodeBuildResult { node, rest: None }
        } else {
            fn construct_non_leaf<T: Aabb>(
                div_axis: impl Axis,
                bots: &mut [T],
            ) -> ConstructResult<T> {
                if bots.is_empty() {
                    return ConstructResult {
                        mid: &mut [],
                        div: None,
                        left: &mut [],
                        right: &mut [],
                    };
                }

                let med_index = bots.len() / 2;
                let (_, med, _) = bots.select_nth_unstable_by(med_index, move |a, b| {
                    crate::util::compare_bots(div_axis, a, b)
                });

                let med_val = med.get().get_range(div_axis).start;

                //It is very important that the median bot end up be binned into the middile bin.
                //We know this must be true because we chose the divider to be the medians left border,
                //and we binned so that all bots who intersect with the divider end up in the middle bin.
                //Very important that if a bots border is exactly on the divider, it is put in the middle.
                //If this were not true, there is no guarantee that the middile bin has bots in it even
                //though we did pick a divider.
                let binned = oned::bin_middle_left_right(div_axis, &med_val, bots);

                ConstructResult {
                    mid: binned.middle,
                    div: Some(med_val),
                    left: binned.left,
                    right: binned.right,
                }
            }

            let rr = match self.axis {
                AxisDyn::X => construct_non_leaf(axgeom::XAXIS, self.bots),
                AxisDyn::Y => construct_non_leaf(axgeom::YAXIS, self.bots),
            };

            let finish_node = NodeFinisher {
                mid: rr.mid,
                div: rr.div,
                axis: self.axis,
                sorter: self.sorter,
                num_elem: rr.left.len() + rr.right.len(),
            };

            let left = rr.left;
            let right = rr.right;

            NodeBuildResult {
                node: finish_node,
                rest: Some([
                    TreeBuildVisitor {
                        bots: left,
                        current_height: self.current_height.saturating_sub(1),
                        sorter: self.sorter,
                        axis: self.axis.next(),
                    },
                    TreeBuildVisitor {
                        bots: right,
                        current_height: self.current_height.saturating_sub(1),
                        sorter: self.sorter,
                        axis: self.axis.next(),
                    },
                ]),
            }
        }
    }

    pub fn recurse_seq(self, res: &mut Vec<Node<'a, T>>) {
        let NodeBuildResult { node, rest } = self.build_and_next();
        res.push(node.finish());
        if let Some([left, right]) = rest {
            left.recurse_seq(res);
            right.recurse_seq(res);
        }
    }
}

struct ConstructResult<'a, T: Aabb> {
    div: Option<T::Num>,
    mid: &'a mut [T],
    right: &'a mut [T],
    left: &'a mut [T],
}

#[derive(Copy, Clone, Default)]
pub struct DefaultSorter;

impl<T: Aabb> Sorter<T> for DefaultSorter {
    fn sort(&self, axis: impl Axis, bots: &mut [T]) {
        crate::util::sweeper_update(axis, bots);
    }
}

#[derive(Copy, Clone, Default)]
pub struct NoSorter;

impl<T: Aabb> Sorter<T> for NoSorter {
    fn sort(&self, _axis: impl Axis, _bots: &mut [T]) {}
}
