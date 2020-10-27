use crate::inner_prelude::*;

#[derive(Copy, Clone)]
pub struct Bot {
    pos: Vec2<f32>,
    num: usize,
}

fn handle_bench_inner(grow: f32, fg: &mut Figure, title: &str, yposition: usize) {
    #[derive(Debug)]
    struct Record {
        num_bots: usize,
        bench_alg: f64,
        bench_par: f64,
        bench_sweep: Option<f64>,
        bench_naive: Option<f64>,
        bench_nosort_par: Option<f64>,
        bench_nosort_seq: Option<f64>,
    }

    let mut records = Vec::new();

    for num_bots in (0..40_000).rev().step_by(500) {
        
          
        let mut bot_inner:Vec<_>=(0..num_bots).map(|a|0isize).collect();
        let mut bots:Vec<  BBox<NotNan<f32>,&mut isize>  > =
            abspiral_f32_nan(grow as f64).zip(bot_inner.iter_mut()).map(|(a,b)|bbox(a,b)).collect();

        let c0 = {
            bench_closure(||{
                let mut tree = broccoli::new_par(&mut bots);

                tree.find_colliding_pairs_mut_par(|a, b| {
                    **a += 1;
                    **b += 1;
                });
    
            })            
        };

        let c1 = {
            
            bench_closure(||{
                let mut tree = broccoli::new(&mut bots);

                tree.find_colliding_pairs_mut(|a, b| {
                    **a += 1;
                    **b += 1;
                });
    
            })
            
        };

        let c3 = {
            if num_bots < 20000 {
                Some(bench_closure(||{
                    broccoli::query::find_collisions_sweep_mut(&mut bots, axgeom::XAXIS, |a, b| {
                        **a -= 2;
                        **b -= 2;
                    });
    
                }))
            } else {
                None
            }
        };

        let c4 = {
            if num_bots < 8000 {
                Some(bench_closure(||{
                    NaiveAlgs::from_slice(&mut bots).find_colliding_pairs_mut(|a, b| {
                        **a -= 1;
                        **b -= 1;
                    });
    
                }))
            } else {
                None
            }
        };

        let c5 = {

            Some(bench_closure(||{
                let mut tree = NotSorted::new_par(&mut bots);

                tree.find_colliding_pairs_mut_par(|a, b| {
                    **a += 1;
                    **b += 1;
                });
    
            }))
            
        };

        let c6 = {
            Some(bench_closure(||{
                let mut tree = NotSorted::new(&mut bots);

                tree.find_colliding_pairs_mut(|a, b| {
                    **a += 1;
                    **b += 1;
                });
    
            }))
            
        };

        for &b in bot_inner.iter().take(8000){
            assert_eq!(b,0)
        }

        records.push(Record {
            num_bots,
            bench_alg: c1,
            bench_par: c0,
            bench_sweep: c3,
            bench_naive: c4,
            bench_nosort_par: c5,
            bench_nosort_seq: c6,
        });
    }

    records.reverse();

    let rects = &mut records;
    use gnuplot::*;
    let x = rects.iter().map(|a| a.num_bots);
    let y1 = rects
        .iter()
        .take_while(|a| a.bench_naive.is_some())
        .map(|a| a.bench_naive.unwrap());
    let y2 = rects
        .iter()
        .take_while(|a| a.bench_sweep.is_some())
        .map(|a| a.bench_sweep.unwrap());
    let y3 = rects.iter().map(|a| a.bench_alg);
    let y4 = rects.iter().map(|a| a.bench_par);
    let y5 = rects
        .iter()
        .take_while(|a| a.bench_nosort_par.is_some())
        .map(|a| a.bench_nosort_par.unwrap());
    let y6 = rects
        .iter()
        .take_while(|a| a.bench_nosort_seq.is_some())
        .map(|a| a.bench_nosort_seq.unwrap());

    fg.axes2d()
        .set_pos_grid(2, 1, yposition as u32)
        .set_title(title, &[])
        .set_legend(Graph(1.0), Graph(1.0), &[LegendOption::Horizontal], &[])
        .lines(
            x.clone(),
            y1,
            &[Caption("Naive"), Color("blue"), LineWidth(2.0)],
        )
        .lines(
            x.clone(),
            y2,
            &[Caption("Sweep and Prune"), Color("green"), LineWidth(2.0)],
        )
        .lines(
            x.clone(),
            y3,
            &[Caption("broccoli Sequential"), Color("red"), LineWidth(2.0)],
        )
        .lines(
            x.clone(),
            y4,
            &[
                Caption("broccoli Parallel"),
                Color("violet"),
                LineWidth(2.0),
            ],
        )
        .lines(
            x.clone(),
            y5,
            &[Caption("KD Tree Parallel"), Color("black"), LineWidth(2.0)],
        )
        .lines(
            x.clone(),
            y6,
            &[
                Caption("KD Tree Sequential"),
                Color("brown"),
                LineWidth(2.0),
            ],
        )
        .set_x_label("Number of Objects", &[])
        .set_y_label("Time taken in seconds", &[]);
}

fn handle_theory_inner(grow: f32, fg: &mut Figure, title: &str, yposition: usize) {
    #[derive(Debug)]
    struct Record {
        num_bots: usize,
        num_comparison_alg: usize,
        num_comparison_naive: Option<usize>,
        num_comparison_sweep: Option<usize>,
        num_comparison_nosort: usize,
    }

    let stop_naive_at = 9_000;
    let stop_sweep_at = 30_000;

    let mut records = Vec::new();

    for num_bots in (0usize..30_000).step_by(500) {
         
        let mut bot_inner:Vec<_>=(0..num_bots).map(|a|0isize).collect();
        let mut bots:Vec<  BBox<NotNan<f32>,&mut isize>  >=abspiral_f32_nan(grow as f64).zip(bot_inner.iter_mut()).map(|(a,b)|bbox(a,b)).collect();

        let c1 = {

            datanum::datanum_test(|maker|{

                let mut tree = broccoli::new(&mut bots);

                tree.find_colliding_pairs_mut(|a, b| {
                    **a += 2;
                    **b += 2;
                });
            })
            
        };

        let c2 = {
            if num_bots < stop_naive_at {

                Some(datanum::datanum_test(|maker|{

                    //let mut bb = bbox_helper::create_bbox_mut(&mut bots, |b| b.rect);

                    NaiveAlgs::from_slice(&mut bots).find_colliding_pairs_mut(|a, b| {
                        **a -= 1;
                        **b -= 1;
                    });
                }))
            } else {
                None
            }
        };
        let c3 = {
            if num_bots < stop_sweep_at {
                Some(datanum::datanum_test(|maker|{
                    //let mut bb = bbox_helper::create_bbox_mut(&mut bots, |b| b.rect);
    
                    broccoli::query::find_collisions_sweep_mut(&mut bots, axgeom::XAXIS, |a, b| {
                        **a -= 1;
                        **b -= 1;
                    });
    
                }))
                
            } else {
                None
            }
        };

        let c4 = {
            
            datanum::datanum_test(|maker|{
                //let mut bb = bbox_helper::create_bbox_mut(&mut bots, |b|b.rect);
    
                let mut tree = NotSorted::new(&mut bots);
    
                tree.find_colliding_pairs_mut(|a, b| {
                    **a += 2;
                    **b += 2;
                });
    
            })
            
        };

        for &a in bot_inner.iter().take(stop_naive_at){
            assert_eq!(a,0);
        }

        let r = Record {
            num_bots,
            num_comparison_alg: c1,
            num_comparison_naive: c2,
            num_comparison_sweep: c3,
            num_comparison_nosort: c4,
        };

        records.push(r);
    }

    let rects = &mut records;
    use gnuplot::*;
    let x = rects.iter().map(|a| a.num_bots);
    let y1 = rects.iter().map(|a| a.num_comparison_alg);
    let y2 = rects
        .iter()
        .take_while(|a| a.num_comparison_naive.is_some())
        .map(|a| a.num_comparison_naive.unwrap());
    let y3 = rects
        .iter()
        .take_while(|a| a.num_comparison_sweep.is_some())
        .map(|a| a.num_comparison_sweep.unwrap());
    let y4 = rects.iter().map(|a| a.num_comparison_nosort);

    fg.axes2d()
        .set_pos_grid(2, 1, yposition as u32)
        .set_title(title, &[])
        .set_legend(Graph(1.0), Graph(1.0), &[LegendOption::Horizontal], &[])
        .lines(
            x.clone(),
            y2,
            &[Caption("Naive"), Color("blue"), LineWidth(4.0)],
        )
        .lines(
            x.clone(),
            y3,
            &[Caption("Sweep and Prune"), Color("green"), LineWidth(2.0)],
        )
        .lines(
            x.clone(),
            y1,
            &[Caption("broccoli"), Color("red"), LineWidth(2.0)],
        )
        .lines(
            x.clone(),
            y4,
            &[Caption("KDTree"), Color("brown"), LineWidth(2.0)],
        )
        .set_x_label("Number of Objects", &[])
        .set_y_label("Number of Comparisons", &[]);
}

pub fn handle_theory(fb: &mut FigureBuilder) {
    let mut fg = fb.build("colfind_theory");

    handle_theory_inner(
        1.0,
        &mut fg,
        "Comparison of space partitioning algs with abspiral(x,1.0)",
        0,
    );
    handle_theory_inner(
        0.05,
        &mut fg,
        "Comparison of space partitioning algs with abspiral(x,0.05)",
        1,
    );
    fb.finish(fg)
}

pub fn handle_bench(fb: &mut FigureBuilder) {
    let mut fg = fb.build("colfind_bench");
    handle_bench_inner(
        1.0,
        &mut fg,
        "Comparison of space partitioning algs with abspiral(x,1.0)",
        0,
    );
    handle_bench_inner(
        0.05,
        &mut fg,
        "Comparison of space partitioning algs with abspiral(x,0.05)",
        1,
    );

    fb.finish(fg);
}
