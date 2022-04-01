use super::*;

///
/// height_seq_fallback: if a subtree has this height, it will be processed as one unit sequentially.
///
pub fn recurse_par<T: Aabb, N: NodeHandler>(
    vistr: CollVis<T, N>,
    prevec: &mut PreVec,
    height_seq_fallback: usize,
    mut func: impl FnMut(HalfPin<&mut T>, HalfPin<&mut T>) + Clone + Send,
) where
    T: Send,
    T::Num: Send,
{
    if vistr.vistr.get_height() <= height_seq_fallback {
        vistr.recurse_seq(prevec, &mut func);
    } else {
        let func2 = func.clone();
        let (n, rest) = vistr.collide_and_next(prevec, func);
        if let Some([left, right]) = rest {
            rayon_core::join(
                || {
                    let (prevec, func) = n.finish();
                    recurse_par(left, prevec, height_seq_fallback, func);
                },
                || {
                    let mut prevec = PreVec::new();
                    recurse_par(right, &mut prevec, height_seq_fallback, func2);
                },
            );
        }
    }
}
