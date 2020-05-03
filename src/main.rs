use competitive::input;

#[cargo_snippet::snippet]
fn main() {
    input! {n};
    let mut ans = 0;
    ans += n;
    println!("{}", ans);
}
