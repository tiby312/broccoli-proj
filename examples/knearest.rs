use axgeom::vec2;
use broccoli::{bbox, prelude::*, rect};

fn distance_squared(a: isize, b: isize) -> isize {
    let a = (a - b).abs();
    a * a
}

fn main() {
    let mut inner1 = vec2(5, 5);
    let mut inner2 = vec2(3, 3);
    let mut inner3 = vec2(7, 7);

    let mut bots = [
        bbox(rect(0, 10, 0, 10), &mut inner1),
        bbox(rect(2, 4, 2, 4), &mut inner2),
        bbox(rect(6, 8, 6, 8), &mut inner3),
    ];

    let mut tree = broccoli::new(&mut bots);

    let mut handler = broccoli::knearest_from_closure(
        &tree,
        (),
        |_, point, a| Some(a.rect.distance_squared_to_point(point).unwrap_or(0)),
        |_, point, a| a.inner.distance_squared_to_point(point),
        |_, point, a| distance_squared(point.x, a),
        |_, point, a| distance_squared(point.y, a),
    );

    let mut res = tree.k_nearest_mut(vec2(30, 30), 2, &mut handler);
    assert_eq!(res.len(), 2);
    assert_eq!(res.total_len(), 2);

    let foo: Vec<_> = res.iter().map(|a| *a[0].bot.inner).collect();

    assert_eq!(foo, vec![vec2(7, 7), vec2(5, 5)])
}
