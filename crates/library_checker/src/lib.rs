//! Verification of [`competitive` crate] by [Library-Checker]
//!
//! [verification summary]
//!
//! [`competitive` crate]: ../competitive/index.html
//! [Library-Checker]: https://judge.yosupo.jp
//! [verification summary]: ?search=verify

pub mod convolution;
pub mod data_structure;
pub mod enumerative_combinatorics;
pub mod graph;
pub mod linear_algebra;
pub mod number_theory;
pub mod other;
pub mod polynomial;
pub mod sample;
pub mod set_power_series;
pub mod string;
pub mod tree;

#[cfg(test)]
mod tests {
    use std::process::Command;

    fn list_verified_problems() -> Vec<(String, String)> {
        let output = Command::new("cargo")
            .args([
                "test",
                "-p",
                "library_checker",
                "--quiet",
                "--",
                "--list",
                "--ignored",
            ])
            .output()
            .expect("Failed to list verified problems")
            .stdout;
        let output = String::from_utf8_lossy(&output);
        output
            .lines()
            .filter_map(|line| {
                let mut split = line.split("::");
                let category = split.next().unwrap();
                if category == "tests" {
                    return None;
                }
                let problem = split.next().unwrap();
                Some((category.to_string(), problem.to_string()))
            })
            .collect()
    }

    #[test]
    fn checklist() {
        let problems = verify::library_checker::get_problem_list().unwrap();
        let verified_problems = list_verified_problems();
        let mut total_count = 0;
        let mut verified_count = 0;
        for (category, problems) in problems {
            println!("{}", category);
            for problem in problems {
                if verified_problems.contains(&(category.clone(), problem.clone())) {
                    println!("  ☑ {}", problem);
                    verified_count += 1;
                } else {
                    println!("  ☐ {}", problem);
                }
                total_count += 1;
            }
        }
        println!(
            "Verified {}/{} problems ({:.2}%)\n",
            verified_count,
            total_count,
            100.0 * verified_count as f64 / total_count as f64
        );
    }

    #[test]
    fn check_correct_category() {
        let problems = verify::library_checker::get_problem_list().unwrap();
        let verified_problems = list_verified_problems();
        let mut failed = vec![];
        for (category, problem) in verified_problems {
            if let Some((correct_category, _)) = problems
                .iter()
                .find(|(_, problems)| problems.contains(&problem))
            {
                if &category != correct_category {
                    println!("{}/{} -> {}", category, problem, correct_category);
                    failed.push((problem, category, correct_category.clone()));
                }
            } else {
                panic!("Problem not found: {} in {:?}", problem, problems);
            }
        }
        assert!(failed.is_empty(), "Some problems are in wrong category");
    }
}
