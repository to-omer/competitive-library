use competitive::prelude::*;
use competitive::{
    algebra::AdditiveOperation, graph::TreeGraphScanner, num::mint_basic::MInt998244353 as M,
    tree::ReRooting,
};

#[verify::library_checker("tree_path_composite_sum")]
pub fn tree_path_composite_sum(reader: impl Read, writer: impl Write) {
    prepare_io!(reader, writer);
    sc!(n, values: [M; n], (graph, edges): @TreeGraphScanner::<usize, (M, M)>::new(n));
    let dp = ReRooting::<(AdditiveOperation<M>, AdditiveOperation<i32>), _>::new_with_inverse(
        &graph,
        |&(sum, count), v, edge| {
            let sum = sum + values[v];
            let count = count + 1;
            if let Some(edge) = edge {
                let (a, b) = edges[edge];
                (a * sum + b * M::new_unchecked(count as u32), count)
            } else {
                (sum, count)
            }
        },
    );
    pp!(@it dp.dp.iter().map(|value| value.0));
}
