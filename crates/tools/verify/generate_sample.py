import sys
from pathlib import Path


def main():
    rootdir = Path(sys.argv[1])
    sys.path.insert(0, str(rootdir))
    from generate import Problem

    problem = Problem(rootdir, Path(sys.argv[2]))
    sample_testcases = set(sys.argv[3:])
    problem.config["tests"] = [
        test for test in problem.config["tests"] if test["name"] in sample_testcases
    ]
    if not problem.config["tests"]:
        raise RuntimeError("sample testcase not found")

    problem.generate_params_h()
    if not problem.is_checker_already_generated():
        problem.compile_checker()
    if not problem.is_testcases_already_generated():
        problem.compile_correct()
        problem.compile_gens()
        problem.make_inputs()
        problem.make_outputs(False)


if __name__ == "__main__":
    main()
