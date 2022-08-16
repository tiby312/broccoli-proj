
### Level Comparison

The below charts show the load balance between the different levels of the tree. 
Tree construction is compared against one call to `find_colliding_pairs`.

Some observations:
* The cost of rebalancing the first level is the most erratic. 
    This is because in some cases we're hitting the worst cases of pdqselect.
	I like to think of the algorithm as a sponge and the problem as water seeping through it.
	First you you have coarse filtering, then it gets more precise.
* The load goes from the top levels to the bottom levels as the aabbs spread out more.
* The load on the first few levels is not high unless the aabbs are clumped up. 
* The leaves don't have much work to do since aabbs have a size, they aren't likely to 
  into a leaf.




<link rel="stylesheet" href="css/poloto.css">

{{#include raw/level_analysis_theory_rebal.svg}}
{{#include raw/level_analysis_theory_query.svg}}
{{#include raw/level_analysis_bench_rebal.svg}}
{{#include raw/level_analysis_bench_query.svg}}

