use super::*;

struct Res {
    rebal: Vec<f32>,
    query: Vec<f32>,
}
impl Res{
    fn new(num_bots: usize, grow_iter: impl Iterator<Item = f64>) -> Vec<(f32,Res)> {
        let mut rects = Vec::new();
        for grow in grow_iter {
            let mut bot_inner: Vec<_> = (0..num_bots).map(|_| 0isize).collect();
            
            let mut bots = distribute(grow, &mut bot_inner, |a| a.to_f32n());
            
            let (mut tree, times1) =
                TreeBuilder::new(&mut bots).build_with_splitter_seq(LevelTimer::new());
            
            
            
            
            let times2 = tree.new_builder().query_with_splitter_seq(
                |a, b| {
                    **a.unpack_inner() += 1;
                    **b.unpack_inner() += 1
                },
                LevelTimer::new()
            );
            
            
            
            
            let t = Res {
                rebal: times1.into_levels().into_iter().map(|x|x as f32).collect(),
                query: times2.into_levels().into_iter().map(|x|x as f32).collect(),
            };
    
            assert_eq!(t.rebal.len(), t.query.len());
    
            assert_eq!(t.rebal.len(), t.query.len());
            rects.push((grow as f32,t))
            
            
        }
        rects
    }
    
}


pub fn handle_bench(fb: &mut FigureBuilder) {
    let num_bots = 5000;
   
    
    let res2 = Res::new(
        num_bots,
        (0..1000).map(|a| {
            let a: f64 = a as f64;
            0.01 + a * 0.00002
        }),
    );

    
    fn draw_graph<'a, I:Iterator<Item=(f32,&'a [f32])>+Clone>(filename:&str,title_name: &str, fb: &mut FigureBuilder, mut it:I,) {
        let mut plot=plotato::plot(title_name,"Spiral Grow","Time taken in Seconds");
        if let Some((xfirst,xrest))=it.next(){
            let num=xrest.len();
            
            let cc = (0..num).map(|ii: usize| it.clone().map(move |(x,a)| [x,a[ii]]));
            
            for (i, y) in cc.enumerate() {
                let s = format!("Level {}", i);
                //let yl = y.clone().map(|_| 0.0);
                plot.line_fill(s,y);
            }
        }
        dbg!("making graph");
        fb.finish_plot(plot,filename);

    }
    
    //let mut fg = fb.build("level_analysis_bench_rebal");
    draw_graph(
        "level_analysis_bench_rebal",
        &format!("Rebal Level Bench with abspiral({},x)", num_bots),
        fb,
        res2.iter().map(|x|(x.0,x.1.rebal.as_slice()))
    );

    draw_graph(
        "level_analysis_bench_query",
        &format!("Query Level Bench with abspiral({},x)", num_bots),
        fb,
        res2.iter().map(|x|(x.0,x.1.query.as_slice()))
    );
}

