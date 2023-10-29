use crate::{app_cache_directory, build_client, gen_case, TestCase};
use serde::Deserialize;
use std::{
    fs::{create_dir, create_dir_all, remove_dir_all},
    time::Duration,
};
use tokio::runtime;

#[derive(Deserialize, Debug)]
struct AOJTestCaseHeader {
    serial: u32,
    name: String,
}
#[derive(Deserialize, Debug)]
struct AOJTestCaseHeaders {
    headers: Vec<AOJTestCaseHeader>,
}

pub fn get_testcases(
    problem_id: &str,
) -> Result<Vec<TestCase>, Box<dyn 'static + std::error::Error>> {
    let mut problemdir = app_cache_directory();
    problemdir.push("aizu-online-judge");
    problemdir.push(problem_id);
    if !problemdir.exists() {
        create_dir_all(&problemdir)?;
    }

    let indir = problemdir.join("in");
    let outdir = problemdir.join("out");
    let is_testcases_already_generated = indir.exists() && outdir.exists();
    if !is_testcases_already_generated {
        if indir.exists() {
            remove_dir_all(&indir)?;
        }
        if outdir.exists() {
            remove_dir_all(&outdir)?;
        }
        create_dir(&indir)?;
        create_dir(&outdir)?;
    }

    let url = format!(
        "https://judgedat.u-aizu.ac.jp/testcases/{}/header",
        problem_id,
    );
    let headers: AOJTestCaseHeaders = build_client()?
        .get(url)
        .timeout(Duration::from_secs(5))
        .send()?
        .json()?;

    let mut cases = Vec::with_capacity(headers.headers.len());
    runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(async {
            let mut tasks = vec![];
            for header in headers.headers {
                let input = indir.join(&header.name).with_extension("in");
                let output = outdir.join(&header.name).with_extension("out");
                cases.push(TestCase {
                    name: header.name.clone(),
                    input: input.clone(),
                    output: output.clone(),
                });
                if !is_testcases_already_generated {
                    let url = format!(
                        "https://judgedat.u-aizu.ac.jp/testcases/{}/{}/in",
                        problem_id, header.serial
                    );
                    tasks.push(tokio::spawn(async move {
                        gen_case(url, input).await.ok();
                    }));
                    let url = format!(
                        "https://judgedat.u-aizu.ac.jp/testcases/{}/{}/out",
                        problem_id, header.serial
                    );
                    tasks.push(tokio::spawn(async move {
                        gen_case(url, output).await.ok();
                    }));
                }
            }
            for task in tasks {
                task.await.ok();
            }
        });
    Ok(cases)
}

#[test]
fn test_itp1_1_a() -> Result<(), Box<dyn 'static + std::error::Error>> {
    let res = get_testcases("ITP1_1_A")?;
    eprintln!("res = {:?}", res);
    Ok(())
}
