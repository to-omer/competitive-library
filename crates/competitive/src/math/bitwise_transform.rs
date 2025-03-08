pub fn bitwise_transform<T, F>(x: &mut [T], mut f: F)
where
    F: FnMut(&mut T, &mut T),
{
    let k = x.len().trailing_zeros() as usize;
    assert!(x.len() == 1 << k);
    for i in 0..k {
        for a in x.chunks_exact_mut(2 << i) {
            let (x, y) = a.split_at_mut(1 << i);
            for (x, y) in x.iter_mut().zip(y.iter_mut()) {
                f(x, y);
            }
        }
    }
}
