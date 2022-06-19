use support::prelude::*;

#[inline(never)]
pub fn bench(max: usize, grow: f64) -> Vec<(i128, Record)> {
    let mut all: Vec<_> = dist::dist(grow).map(|x| Dummy(x, 0u32)).take(max).collect();

    (0..max)
        .step_by(100)
        .skip(1)
        .map(|a| {
            let bots = &mut all[0..a];
            (a as i128, new_record(bots))
        })
        .collect()
}

#[derive(Debug)]
pub struct Record {
    pub tree: (f64, f64),
    pub par_tree: (f64, f64),
    pub nosort: (f64, f64),
    //pub par_nosort:(f64,f64)
}

pub fn new_record<T: ColfindHandler>(bots: &mut [T]) -> Record
where
    T: Send,
    T::Num: Send,
{
    let mut recorder = Bencher;
    let (mut tree, par_tree1) = recorder.time_ext(|| broccoli::Tree::par_new(bots));

    let par_tree2 = recorder.time(|| {
        tree.par_find_colliding_pairs(T::handle);
    });

    let (mut tree, tree1) = recorder.time_ext(|| broccoli::Tree::new(bots));

    let tree2 = recorder.time(|| {
        tree.find_colliding_pairs(T::handle);
    });

    /*
    let (mut tree,par_notree1) = recorder.time_ext(|| {
        broccoli::NotSortedTree::par_new(bots)
    });

    let par_notree2=recorder.time(||{
        tree.par_find_colliding_pairs(T::handle);
    });
    */

    let (mut tree, notree1) = recorder.time_ext(|| broccoli::NotSortedTree::new(bots));

    let notree2 = recorder.time(|| {
        tree.find_colliding_pairs(T::handle);
    });

    Record {
        tree: (tree1, tree2),
        par_tree: (par_tree1, par_tree2),
        nosort: (notree1, notree2),
        //par_nosort:(par_notree1,par_notree2)
    }
}
