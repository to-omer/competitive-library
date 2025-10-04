use competitive::prelude::*;
#[doc(no_inline)]
pub use competitive::{
    algebra::{LinearOperation, ReverseOperation},
    data_structure::SegmentTree,
    graph::TreeGraphScanner,
    num::{MInt, mint_basic::MInt998244353},
    tree::HeavyLightDecomposition,
};

#[verify::library_checker("vertex_set_path_composite")]
pub fn vertex_set_path_composite(reader: impl Read, mut writer: impl Write) {
    let s = read_all_unchecked(reader);
    let mut scanner = Scanner::new(&s);
    scan!(scanner, n, q, ab: [(MInt998244353, MInt998244353); n], (mut graph, _): @TreeGraphScanner::<usize, ()>::new(n));
    let hld = HeavyLightDecomposition::new(0, &mut graph);
    let mut nab = vec![(MInt998244353::default(), MInt998244353::default()); n];
    for i in 0..n {
        nab[hld.vidx[i]] = ab[i];
    }
    let mut seg1 = SegmentTree::<LinearOperation<_>>::from_vec(nab.clone());
    let mut seg2 = SegmentTree::<ReverseOperation<LinearOperation<_>>>::from_vec(nab);
    for _ in 0..q {
        match scanner.scan::<usize>() {
            0 => {
                scan!(scanner, p, cd: (MInt998244353, MInt998244353));
                seg1.set(hld.vidx[p], cd);
                seg2.set(hld.vidx[p], cd);
            }
            1 => {
                scan!(scanner, u, v, x: MInt998244353);
                let (a, b) = hld.query_noncom::<LinearOperation<_>, _, _>(
                    u,
                    v,
                    false,
                    |l, r| seg1.fold(l..r),
                    |l, r| seg2.fold(l..r),
                );
                writeln!(writer, "{}", a * x + b).ok();
            }
            _ => unreachable!("unknown query"),
        }
    }
}
