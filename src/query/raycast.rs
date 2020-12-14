//!
//! # User guide
//!
//! There are four flavors of the same fundamental raycast api provided in this module.
//! There is a naive version, and there is a version that uses the tree, and there are mutable versions of those
//! that return mutable references.
//!
//!
//! In addition to the tree, the user provides the geometric functions needed by passing an implementation of RayCast.
//! The user must also provide a rectangle within which all objects that the user is interested in possibly
//! being hit by the raycast must include.
//!
//! What is returned is the distance to where the ray cast stopped, plus a list of all bots at that distance.
//! In most cases, only one object is returned, but in the cases where they are ties more can be returned.
//! All possible solutions are returned since it would be hard to define which of the tied objects would be returned.
//! So the Option returns Some() if and only if the list returned has atleast one element in it.
//!
//! # Notes

//! At first the algorithm worked by splitting the ray into two where the ray intersected the divider.
//! So one ray would have the same origin point, and the other would have the point at which the ray
//! intersected the divder as the origin point. The problem with this is that there might not be a clean solution
//! to the new point of the second ray. The point that you compute may not lie exactly on a point along the ray.
//!
//! With real numbers this isnt a problem. There would always be a solution. But real numbers don't exist
//! in the real world. Floating points will be close, but not perfect. If you are using integers, the corner case problems
//! are more apparent.
//!
//! The solution instead was to never subdivide the ray. Its always the same. Instead, keep subdividing the area into rectangles.
//!
//! Why does the user have to provide a finite rectangle up front? The reason is implementation simplicity/performance.
//! By doing this, we don't have to special case the nodes along the outside of the tree.
//! We also don't have have to worry about overflow and underflow problems of providing a rectangle that
//! just barely fits into the number type.
//!

use crate::query::inner_prelude::*;
use axgeom::Ray;
use core::cmp::Ordering;

///A Vec<T> is returned since there coule be ties where the ray hits multiple T at a length N away.
//pub type RayCastResult<T, N> = axgeom::CastResult<(Vec<T>, N)>;

///This is the trait that defines raycast specific geometric functions that are needed by this raytracing algorithm.
///By containing all these functions in this trait, we can keep the trait bounds of the underlying Num to a minimum
///of only needing Ord.
pub trait RayCast {
    type T: Aabb<Num = Self::N>;
    type N: Num;

    ///Returns true if the ray intersects with this rectangle.
    ///This function allows as to prune which nodes to visit.
    fn compute_distance_to_rect(
        &mut self,
        ray: &Ray<Self::N>,
        a: &Rect<Self::N>,
    ) -> axgeom::CastResult<Self::N>;

    ///The expensive collision detection
    ///This is where the user can do expensive collision detection on the shape
    ///contains within it's bounding box.
    ///Its default implementation just calls compute_distance_to_rect()
    fn compute_distance_to_bot(
        &mut self,
        ray: &Ray<Self::N>,
        a: &Self::T,
    ) -> axgeom::CastResult<Self::N> {
        self.compute_distance_to_rect(ray, a.get())
    }
}

pub struct RayCastClosure<'a, A, B, C, T> {
    pub a: &'a mut A,
    pub broad: B,
    pub fine: C,
    pub _p: PhantomData<T>,
}

impl<
        'a,
        A,
        B: FnMut(&mut A, &Ray<T::Num>, &Rect<T::Num>) -> CastResult<T::Num>,
        C: FnMut(&mut A, &Ray<T::Num>, &T) -> CastResult<T::Num>,
        T: Aabb,
    > RayCastClosure<'a, A, B, C, T>
{
    pub fn new(acc: &'a mut A, broad: B, fine: C) -> Self {
        RayCastClosure {
            a: acc,
            broad,
            fine,
            _p: PhantomData,
        }
    }
}
impl<
        A,
        B: FnMut(&mut A, &Ray<T::Num>, &Rect<T::Num>) -> CastResult<T::Num>,
        C: FnMut(&mut A, &Ray<T::Num>, &T) -> CastResult<T::Num>,
        T: Aabb,
    > RayCast for &mut RayCastClosure<'_, A, B, C, T>
{
    type T = T;
    type N = T::Num;
    fn compute_distance_to_rect(
        &mut self,
        ray: &Ray<Self::N>,
        a: &Rect<Self::N>,
    ) -> CastResult<Self::N> {
        (self.broad)(&mut self.a, ray, a)
    }

    fn compute_distance_to_bot(&mut self, ray: &Ray<Self::N>, a: &Self::T) -> CastResult<Self::N> {
        (self.fine)(&mut self.a, ray, a)
    }
}

fn make_rect_from_range<A: Axis, N: Num>(axis: A, range: &Range<N>, rect: &Rect<N>) -> Rect<N> {
    if axis.is_xaxis() {
        Rect {
            x: *range,
            y: rect.y,
        }
    } else {
        Rect {
            x: rect.x,
            y: *range,
        }
    }
}

struct Closest<'a, T: Aabb> {
    closest: Option<(Vec<PMut<'a, T>>, T::Num)>,
}
impl<'a, T: Aabb> Closest<'a, T> {
    fn consider<R: RayCast<N = T::Num, T = T>>(
        &mut self,
        ray: &Ray<T::Num>,
        b: PMut<'a, T>,
        raytrait: &mut R,
    ) {
        //first check if bounding box could possibly be a candidate.
        let y = match raytrait.compute_distance_to_rect(ray, b.get()) {
            axgeom::CastResult::Hit(val) => val,
            axgeom::CastResult::NoHit => {
                return;
            }
        };

        match self.closest.as_mut() {
            Some(dis) => {
                if y>dis.1{
                    //no way this bot will be a candidate, return.
                    return;
                }else{
                    //this aabb could be a candidate, continue.   
                }
                /*
                match y.cmp(&dis.1) {
                    Ordering::Greater => {
                        //no way this bot will be a candidate, return.
                        return;
                    }
                    Ordering::Less => {
                        //this aabb could be a candidate, continue.
                    }
                    Ordering::Equal => {
                        //this aabb could be a candidate, continue.
                    }
                }
                */
            }
            None => {
                //this aabb could be a candidate, continue,
            }
        }

        let x = match raytrait.compute_distance_to_bot(ray, &b) {
            axgeom::CastResult::Hit(val) => val,
            axgeom::CastResult::NoHit => {
                return;
            }
        };

        match self.closest.as_mut() {
            Some(mut dis) => {
                if x>dis.1{
                    //do nothing
                }else{
                    dis.0.clear();
                    dis.0.push(b);
                    dis.1 = x;
                }
                /*
                match x.cmp(&dis.1) {
                    Ordering::Greater => {
                        //dis
                        //do nothing.
                    }
                    Ordering::Less => {
                        dis.0.clear();
                        dis.0.push(b);
                        dis.1 = x;
                    }
                    Ordering::Equal => {
                        dis.0.push(b);
                    }
                }
                */
            }
            None => self.closest = Some((vec![b], x)),
        };
    }

    fn get_dis(&self) -> Option<T::Num> {
        match &self.closest {
            Some(x) => Some(x.1),
            None => None,
        }
    }
}

struct Blap<'a, R: RayCast> {
    rtrait: R,
    ray: Ray<R::N>,
    closest: Closest<'a, R::T>,
}
impl<'a, R: RayCast> Blap<'a, R> {
    fn should_handle_rect(&mut self, rect: &Rect<R::N>) -> bool {
        match self.rtrait.compute_distance_to_rect(&self.ray, rect) {
            axgeom::CastResult::Hit(val) => match self.closest.get_dis() {
                Some(dis) => {
                    if val <= dis {
                        return true;
                    }
                }
                None => {
                    return true;
                }
            },
            axgeom::CastResult::NoHit => {}
        }
        false
    }
}

//Returns the first object that touches the ray.
fn recc<'a, 'b: 'a, A: Axis, T: Aabb, R: RayCast<N = T::Num, T = T>>(
    axis: A,
    stuff: LevelIter<VistrMut<'a, NodeMut<'b, T>>>,
    rect: Rect<T::Num>,
    blap: &mut Blap<'a, R>,
) {
    let ((_depth, nn), rest) = stuff.next();
    //let nn = nn.get_mut();
    match rest {
        Some([left, right]) => {
            let axis_next = axis.next();

            let div = match nn.div {
                Some(b) => b,
                None => return,
            };

            let (rleft, rright) = rect.subdivide(axis, div);

            let range = &match nn.cont {
                Some(range) => range,
                None => Range {
                    start: div,
                    end: div,
                },
            };

            let rmiddle = make_rect_from_range(axis, range, &rect);

            match blap.ray.range_side(axis, range) {
                Ordering::Less => {
                    if blap.should_handle_rect(&rleft) {
                        recc(axis_next, left, rleft, blap);
                    }

                    if blap.should_handle_rect(&rmiddle) {
                        for b in nn.into_range().iter_mut() {
                            blap.closest.consider(&blap.ray, b, &mut blap.rtrait);
                        }
                    }

                    if blap.should_handle_rect(&rright) {
                        recc(axis_next, right, rright, blap);
                    }
                }
                Ordering::Greater => {
                    if blap.should_handle_rect(&rright) {
                        recc(axis_next, right, rright, blap);
                    }

                    if blap.should_handle_rect(&rmiddle) {
                        for b in nn.into_range().iter_mut() {
                            blap.closest.consider(&blap.ray, b, &mut blap.rtrait);
                        }
                    }

                    if blap.should_handle_rect(&rleft) {
                        recc(axis_next, left, rleft, blap);
                    }
                }
                Ordering::Equal => {
                    //Assume there are more elements in the children than the current node,
                    //so recurse first.
                    if blap.should_handle_rect(&rleft) {
                        recc(axis_next, left, rleft, blap);
                    }

                    if blap.should_handle_rect(&rright) {
                        recc(axis_next, right, rright, blap);
                    }
                    if blap.should_handle_rect(&rmiddle) {
                        for b in nn.into_range().iter_mut() {
                            blap.closest.consider(&blap.ray, b, &mut blap.rtrait);
                        }
                    }
                }
            }
        }
        None => {
            //Can't do better here since for leafs, cont is none.
            for b in nn.into_range().iter_mut() {
                blap.closest.consider(&blap.ray, b, &mut blap.rtrait);
            }
        }
    }
}

pub use self::mutable::raycast_mut;
pub use self::mutable::raycast_naive_mut;

mod mutable {
    use super::*;

    pub fn raycast_naive_mut<'a, T: Aabb>(
        bots: PMut<'a, [T]>,
        ray: Ray<T::Num>,
        mut rtrait: impl RayCast<N = T::Num, T = T>,
        border: Rect<T::Num>,
    ) -> axgeom::CastResult<(Vec<PMut<'a, T>>, T::Num)> {
        let mut closest = Closest { closest: None };

        for b in bots.iter_mut() {
            if border.intersects_rect(b.get()) {
                closest.consider(&ray, b, &mut rtrait);
            }
        }

        match closest.closest {
            Some((a, b)) => axgeom::CastResult::Hit((a, b)),
            None => axgeom::CastResult::NoHit,
        }
    }

    pub fn raycast_mut<'a, 'b: 'a, A: Axis, T: Aabb>(
        axis: A,
        vistr: VistrMut<'a, NodeMut<'b, T>>,
        rect: Rect<T::Num>,
        ray: Ray<T::Num>,
        rtrait: impl RayCast<N = T::Num, T = T>,
    ) -> axgeom::CastResult<(Vec<PMut<'a, T>>, T::Num)> {
        let dt = vistr.with_depth(Depth(0));

        let closest = Closest { closest: None };
        let mut blap = Blap {
            rtrait,
            ray,
            closest,
        };
        recc(axis, dt, rect, &mut blap);

        match blap.closest.closest {
            Some((a, b)) => axgeom::CastResult::Hit((a, b)),
            None => axgeom::CastResult::NoHit,
        }
    }
}
