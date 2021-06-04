use crate::support::prelude::*;

use duckduckgeo;

#[derive(Copy, Clone)]
pub struct Bot {
    id: usize, //id used to verify pairs against naive
    pos: Vec2<f32>,
    vel: Vec2<f32>,
    force: Vec2<f32>,
}

impl Bot {
    fn update(&mut self) {
        self.vel += self.force;
        //non linear drag
        self.vel *= 0.9;
        self.pos += self.vel;
        self.force = vec2same(0.0);
    }
}

pub fn make_demo(dim: Rect<f32>) -> Demo {
    let radius = 5.0;

    let mut bots = {
        let mut idcounter = 0;
        support::make_rand(4000, dim, |pos| {
            let b = Bot {
                id: idcounter,
                pos,
                vel: vec2same(0.0),
                force: vec2same(0.0),
            };
            idcounter += 1;
            b
        })
    };

    Demo::new(move |cursor, canvas, check_naive| {
        for b in bots.iter_mut() {
            b.update();
        }

        let mut base = broccoli::container::TreeIndBase::new(&mut bots, |a| {
            Rect::from_point(a.pos, vec2same(radius))
        });
        let mut tree = base.build();

        tree.for_all_not_in_rect_mut(&dim, |a| {
            let a = a.unpack_inner();
            duckduckgeo::collide_with_border(&mut a.pos, &mut a.vel, &dim, 0.5);
        });

        let vv = vec2same(100.0);
        tree.for_all_in_rect_mut(&axgeom::Rect::from_point(cursor, vv), |b| {
            let b = b.unpack_inner();
            let _ = duckduckgeo::repel_one(b.pos, &mut b.force, cursor, 0.001, 20.0);
        });

        //Draw the dividers
        let mut rects = canvas.rects();
        let mut lines = canvas.lines(2.0);
        tree.draw_divider(
            |axis, node, rect, _| {
                let mid = if let Some(div) = node.div {
                    axis.map(
                        |x| get_nonleaf_mid(x, rect, div),
                        |y| get_nonleaf_mid(y, rect, div),
                    )
                } else {
                    get_leaf_mid(rect)
                };

                if !node.range.is_empty() {
                    rects.add(
                        axis.map_val(
                            Rect {
                                x: node.cont.into(),
                                y: rect.y.into(),
                            },
                            Rect {
                                x: rect.x.into(),
                                y: node.cont.into(),
                            },
                        )
                        .into(),
                    );
                }

                for b in node.range.iter() {
                    lines.add(b.inner.pos.into(), mid.into());
                }
            },
            dim,
        );

        rects
            .send_and_uniforms(canvas)
            .with_color([0.0, 1.0, 1.0, 0.3])
            .draw();

        lines
            .send_and_uniforms(canvas)
            .with_color([1.0, 0.5, 1.0, 0.6])
            .draw();

        tree.find_colliding_pairs_mut_par(RayonJoin, |a, b| {
            let (a, b) = (a.unpack_inner(), b.unpack_inner());
            let _ = duckduckgeo::repel([(a.pos, &mut a.force), (b.pos, &mut b.force)], 0.001, 2.0);
        });

        if check_naive {
            broccoli::assert::assert_query(&mut tree);
        }

        let mut circles = canvas.circles();
        for bot in bots.iter() {
            circles.add(bot.pos.into());
        }
        circles
            .send_and_uniforms(canvas, radius)
            .with_color([1.0, 1.0, 0.0, 0.6])
            .draw();

        let mut lines = canvas.lines(radius * 0.5);
        for bot in bots.iter() {
            lines.add(
                bot.pos.into(),
                (bot.pos + vec2(0.0, (bot.id % 100) as f32) * 0.1).into(),
            );
        }
        lines
            .send_and_uniforms(canvas)
            .with_color([0.0, 0.0, 1.0, 0.5])
            .draw();
    })
}

fn get_leaf_mid(rect: &Rect<f32>) -> Vec2<f32> {
    let ((x1, x2), (y1, y2)) = rect.get();
    let midx = x1 + (x2 - x1) / 2.0;

    let midy = y1 + (y2 - y1) / 2.0;
    vec2(midx, midy)
}

fn get_nonleaf_mid(axis: impl Axis, rect: &Rect<f32>, div: f32) -> Vec2<f32> {
    let ((x1, x2), (y1, y2)) = rect.get();
    let midx = if !axis.is_xaxis() {
        x1 + (x2 - x1) / 2.0
    } else {
        div
    };

    let midy = if axis.is_xaxis() {
        y1 + (y2 - y1) / 2.0
    } else {
        div
    };
    vec2(midx, midy)
}
