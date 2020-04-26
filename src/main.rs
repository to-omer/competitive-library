use cargo_snippet::snippet;
use competitive_library::input;

#[snippet]
fn main() {
    input! {n};
    let mut ans = 0;
    ans += n;
    println!("{}", ans);
}
