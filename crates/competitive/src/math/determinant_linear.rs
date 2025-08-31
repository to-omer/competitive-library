use super::{
    AddMulOperation, Fps998244353, Matrix, MemorizedFactorial, One, Xorshift, Zero,
    montgomery::MInt998244353 as M,
};

pub fn determinant_linear(mut m0: Vec<Vec<M>>, m1: Vec<Vec<M>>) -> Option<Vec<M>> {
    let mut rng = Xorshift::new();
    let a = M::new_unchecked(rng.rand(M::get_mod() as _) as _);
    let n = m0.len();
    for i in 0..n {
        for j in 0..n {
            m0[i][j] += m1[i][j] * a;
        }
    }
    let mut f = determinant_linear_non_singular(m1, m0)?;
    f.reverse();
    let mf = MemorizedFactorial::new(n + 1);
    Some(Fps998244353::from_vec(f).taylor_shift(-a, &mf).data)
}

fn determinant_linear_non_singular(mut m0: Vec<Vec<M>>, mut m1: Vec<Vec<M>>) -> Option<Vec<M>> {
    let n = m0.len();
    let mut f = M::one();
    for d in 0..n {
        let i = m1.iter().position(|m1| !m1[d].is_zero())?;
        if i != d {
            m0.swap(i, d);
            m1.swap(i, d);
            f = -f;
        }
        f *= m1[d][d];
        let r = m1[d][d].inv();
        for j in 0..n {
            m0[d][j] *= r;
            m1[d][j] *= r;
        }
        assert!(m1[d][d].is_one());
        for i in d + 1..n {
            let a = m1[i][d];
            for k in 0..n {
                m0[i][k] = m0[i][k] - a * m0[d][k];
                m1[i][k] = m1[i][k] - a * m1[d][k];
            }
        }
        for j in d + 1..n {
            let a = m1[d][j];
            for k in 0..n {
                m0[k][j] = m0[k][j] - a * m0[k][d];
                m1[k][j] = m1[k][j] - a * m1[k][d];
            }
        }
    }
    for m0 in m0.iter_mut() {
        for m0 in m0.iter_mut() {
            *m0 = -*m0;
        }
    }
    let mut p = Matrix::<AddMulOperation<_>>::from_vec(m0).characteristic_polynomial();
    for p in p.iter_mut() {
        *p *= f;
    }
    Some(p)
}
