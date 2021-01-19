use broccoli::prelude::*;
fn main() {
    let mut aabbs = [
        broccoli::bbox(broccoli::rect(0isize, 10, 0, 10), 0),
        broccoli::bbox(broccoli::rect(15, 20, 15, 20), 1),
        broccoli::bbox(broccoli::rect(5, 15, 5, 15), 2),
    ];

    let mut base=broccoli::container::TreeIndBase::new(&mut aabbs,|a|a.rect);
    let mut tree=base.build();

    //Find all colliding aabbs.
    let mut pairs = tree.collect_colliding_pairs(|a, b| {
        a.inner += 1;
        b.inner += 1;
        Some(())
    });

    //Collect all evens
    let mut evens = tree.collect_all(|_, b| if b.inner % 2 == 0 { Some(()) } else { None });

    //We can iterate over all the colliding pairs as well as our custom group
    //multiple times without having to query the tree over and over again.
    for _ in 0..2 {
        //mutate our custom group
        for (a, ()) in evens.get_mut(&mut aabbs).iter_mut() {
            a.inner += 1;
        }

        //mutate every colliding pair.
        pairs.for_every_pair_mut(&mut aabbs, |a, b, ()| {
            a.inner += 1;
            b.inner += 1;
        })
    }

    assert_eq!(aabbs[0].inner, 3);
    assert_eq!(aabbs[1].inner, 1);
    assert_eq!(aabbs[2].inner, 5);
}
