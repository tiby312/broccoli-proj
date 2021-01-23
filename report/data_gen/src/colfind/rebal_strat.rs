use crate::inner_prelude::*;

pub fn handle(fb: &mut FigureBuilder) {
    handle_num_bots(fb, 0.2);
}

#[derive(Debug)]
struct Record {
    num_bots: usize,
    arr: [f64; 4],
}
impl Record {
    fn draw(records: &[Record], fg: &mut Figure) {
        const NAMES: &[&str] = &[
            "RebalStrat Checked Par",
            "RebalStrat Not Checked Par",
            "RebalStrat Checked Seq",
            "RebalStrat Not Checked Seq",
        ];
        {
            let k = fg
                .axes2d()
                .set_title(
                    &"Checked vs Unchecked binning indexing with abspiral(x,1.0)".to_string(),
                    &[],
                )
                .set_legend(Graph(1.0), Graph(1.0), &[LegendOption::Horizontal], &[])
                .set_x_label("Number of Objects", &[])
                .set_y_label("Time in Seconds", &[]);

            let x = records.iter().map(|a| a.num_bots);
            for index in 0..4 {
                let y = records.iter().map(|a| a.arr[index]);
                k.lines(
                    x.clone(),
                    y,
                    &[Caption(NAMES[index]), Color(COLS[index]), LineWidth(2.0)],
                );
            }
        }
    }
}

fn handle_num_bots(fb: &mut FigureBuilder, grow: f64) {
    let mut rects = Vec::new();

    for num_bots in (0..700_000).step_by(5000) {
        let mut bot_inner: Vec<_> = (0..num_bots).map(|_| 0isize).collect();

        let arr = [
            {
                let mut scene = distribute(grow, &mut bot_inner, |a| a.to_f32n());

                bench_closure(|| {
                    let tree = TreeBuilder::new(&mut scene)
                        .with_bin_strat(BinStrat::Checked)
                        .build_par(RayonJoin);

                    black_box(tree);
                })
            },
            {
                let mut scene = distribute(grow, &mut bot_inner, |a| a.to_f32n());

                bench_closure(|| {
                    let tree = TreeBuilder::new(&mut scene)
                        .with_bin_strat(BinStrat::NotChecked)
                        .build_par(RayonJoin);

                    black_box(tree);
                })
            },
            {
                let mut scene = distribute(grow, &mut bot_inner, |a| a.to_f32n());

                bench_closure(|| {
                    let tree = TreeBuilder::new(&mut scene)
                        .with_bin_strat(BinStrat::Checked)
                        .build_seq();

                    black_box(tree);
                })
            },
            {
                let mut scene = distribute(grow, &mut bot_inner, |a| a.to_f32n());

                bench_closure(|| {
                    let tree = TreeBuilder::new(&mut scene)
                        .with_bin_strat(BinStrat::NotChecked)
                        .build_seq();

                    black_box(tree);
                })
            },
        ];

        let r = Record { num_bots, arr };
        rects.push(r);
    }

    let mut fg = fb.build("checked_vs_unchecked_binning");

    Record::draw(&rects, &mut fg);

    fb.finish(fg);
}
