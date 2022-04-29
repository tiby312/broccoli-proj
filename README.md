### Broccoli

[![Crates.io](https://img.shields.io/crates/v/broccoli)](https://crates.io/crates/broccoli)
[![docs.rs](https://docs.rs/broccoli/badge.svg)](https://docs.rs/broccoli)
[![Crates.io](https://img.shields.io/crates/d/broccoli)](https://crates.io/crates/broccoli)

Broccoli is a broad-phase collision detection library. 

The base data structure is a hybrid between a [KD Tree](https://en.wikipedia.org/wiki/K-d_tree) and [Sweep and Prune](https://en.wikipedia.org/wiki/Sweep_and_prune).

Checkout it out on [github](https://github.com/tiby312/broccoli) and on [crates.io](https://crates.io/crates/broccoli). Documentation at [docs.rs](https://docs.rs/broccoli). For a report on the algorithm used as well as analysis on the crate in general, see the [broccoli book](https://tiby312.github.io/broccoli_report).

### Screenshot

Screen capture from the inner `demo` project.

<img src="./assets/screenshot.gif" alt="screenshot">

### Other crates

The [broccoli-ext](https://crates.io/crates/broccoli-ext) crate contains some more functionality but relies on unsafe.

### Example

```rust
use broccoli::prelude::*;
use broccoli::tree::rect;
fn main() {
    let mut inner1 = 0;
    let mut inner2 = 0;
    let mut inner3 = 0;

    //Rect is stored directly in tree,
    //but inner is not.
    let mut aabbs = [
        (rect(00, 10, 00, 10), &mut inner1),
        (rect(15, 20, 15, 20), &mut inner2),
        (rect(05, 15, 05, 15), &mut inner3),
    ];

    //This will change the order of the elements
    //in bboxes,but this is okay since we
    //populated it with mutable references.
    let mut tree = broccoli::tree::new(&mut aabbs);

    //Find all colliding aabbs.
    tree.colliding_pairs(|a, b| {
        **a.unpack_inner() += 1;
        **b.unpack_inner() += 1;
    });

    assert_eq!(inner1, 1);
    assert_eq!(inner2, 1);
    assert_eq!(inner3, 2);
}
```


### Name

If you shorten "broad-phase collision" to "broad colli" and say it fast, it sounds like broccoli.
Broccoli are also basically small trees and broccoli uses a tree data structure.