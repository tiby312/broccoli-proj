### Broccoli

[![Crates.io](https://img.shields.io/crates/v/broccoli)](https://crates.io/crates/broccoli)
[![docs.rs](https://docs.rs/broccoli/badge.svg)](https://docs.rs/broccoli)
[![Crates.io](https://img.shields.io/crates/d/broccoli)](https://crates.io/crates/broccoli)

Broccoli is a broad-phase collision detection library. 

The base data structure is a hybrid between a [KD Tree](https://en.wikipedia.org/wiki/K-d_tree) and [Sweep and Prune](https://en.wikipedia.org/wiki/Sweep_and_prune).

Checkout it out on [github](https://github.com/tiby312/broccoli-project) and on [crates.io](https://crates.io/crates/broccoli). Documentation at [docs.rs](https://docs.rs/broccoli). 
### Screenshot

Screen capture from the inner `demo` project.

<img src="./assets/screenshot.gif" alt="screenshot">


### Example

```rust
use broccoli::rect;
fn main() {
    let mut inner1 = 0;
    let mut inner2 = 0;
    let mut inner3 = 0;

    // Rect is stored directly in tree,
    // but inner is not.
    let mut aabbs = [
        (rect(00, 10, 00, 10), &mut inner1),
        (rect(15, 20, 15, 20), &mut inner2),
        (rect(05, 15, 05, 15), &mut inner3),
    ];

    // Construct tree by doing many swapping of elements
    let mut tree = broccoli::Tree::new(&mut aabbs);

    // Find all colliding aabbs.
    tree.find_colliding_pairs(|a, b| {
        // We aren't given &mut T reference, but instead of AabbPin<&mut T>.
        // We call unpack_inner() to extract the portion that we are allowed to mutate.
        // (We are not allowed to change the bounding box while in the tree)
        **a.unpack_inner() += 1;
        **b.unpack_inner() += 1;
    });

    assert_eq!(inner1, 1);
    assert_eq!(inner2, 1);
    assert_eq!(inner3, 2);
}
```


 ### Size of `T` in `Tree`

 During construction, the elements of a tree are swapped around a lot. Therefore if the size
 of T is too big, the performance can regress a lot! To combat this, consider using the semi-direct
 or even indirect layouts listed below. The Indirect layout achieves the smallest element size (just one pointer),
 however it can suffer from a lot of cache misses of large problem sizes. The Semi-direct layout
 is more cache-friendly but can use more memory. See more in the optimizations section below.
 In almost all cases you want to use the Semi-direct layout.

 - `(Rect<N>,&mut T)` Semi-direct
 - `(Rect<N>,T)` Direct
 - `&mut (Rect<N>,T)` Indirect

 I made the [`ManySwap`] marker trait to help bring awareness to this performance regression trap.
 It is implemented on a lot of types that are guaranteed to be small.
 If you know what you are doing you can use the [`ManySwappable`] wrapper struct that automatically
 implements that trait, or implement it yourself on your own type.

 You can also construct a Tree using Semi-direct or indirect, and then convert it to direct. (See
 the [`Tree::from_tree_data()`] function.) However, I'm not sure if there are performance benefits to this.

 ### Parallelism

 **WARNING**: Heterogenous cpus are becoming popular where you might have some high power cores and some low power cores. To get consistent performance on a system, you will have to set the thread affinity to make rayon's
 threadpools only run on one group type. This makes writing system independent code very hard. Consider sticking
 to single threaded unless you are able to tweak the parallel performance. The gains from simply using the broccoli algorithm
 dominate over the gains for making it parallel, so just using broccoli but sticking to sequential might be enough
 for your usecase.
 
 Parallel versions of construction and colliding pair finding functions
 are provided. They use [rayon](https://crates.io/crates/rayon) under the hood which uses work stealing to
 parallelize divide and conquer style recursive functions.

 ### Floating Point

 Broccoli only requires `PartialOrd` for its number type. Instead of panicking on comparisons
 it doesn't understand, it will just arbitrary pick a result. So if you use regular float primitive types
 and there is even just one `NaN`, tree construction and querying will not panic,
 but would have unspecified results.
 If using floats, it's the users responsibility to not pass `NaN` values into the tree.
 There is no static protection against this, though if this is desired you can use
 the [ordered-float](https://crates.io/crates/ordered-float) crate. The Ord trait was not
 enforced to give users the option to use primitive floats directly which can be easier to
 work with.

 ### Protecting Invariants Statically

 A lot is done to forbid the user from violating the invariants of the tree once constructed
 while still allowing them to mutate parts of each element of the tree. The user can mutably traverse
 the tree but the mutable references returns are hidden behind the `AabbPin<T>` type that forbids
 mutating the aabbs.

 ### Do I have to rebuild the tree every time?

 Yes. I optimized for fast querying over fast building. I noticed that building times are consistent while
 querying times can vary wildly depending on how many are overlapping, and that querying times dominate
 over rebuilding times after a certain number of collisions. I think a lot of collisions systems
 have mechanisms to not have to rebuild the entire tree, but do so at the cost of slower querying times. i.e.
 they may insert loose bounding boxes which would increase the number of false positives during querying.
 These systems are great if you know up front that you will never have that many collisions.
 However in other systems you might not have a bound on that so, broccoli was optimized for situations
 where the number of collisions could dominate.  

### Optimisation

I've focused mainly on making finding colliding pairs as fast as possible primarily in
distributions where there are a lot of overlapping aabbs.

Quick rundown of what i've spent effort on and a rough estimate of performance
cost of each algorithm in general. 

| Algorithm        | Cost | Effort spent  |
| ---------------- | ---- | ------------- |
| Construction     |   7  |        10     |
| Colliding Pairs  |   8  |        10     |
| Collide With     |   3  |         2     |
| knearest         |   1  |         2     |
| raycast          |   1  |         2     |
| rect             |   1  |         2     |
| nbody            |  10  |         1     |

Numbers are out of 10 and are just rough made up numbers. For more in-depth analysis, see the
output of the inner `analysis/report-web/plot-gen` at:
[https://tiby312.github.io/broccoli_plots/](https://tiby312.github.io/broccoli_plots/)

See legacy report (I havent updated it in a while) from `analysis/report-legacy` at:
[broccoli book](https://tiby312.github.io/broccoli_report).


### Name

If you shorten "broad-phase collision" to "broad colli" and say it fast, it sounds like broccoli.
Broccoli are also basically small trees and broccoli uses a tree data structure.