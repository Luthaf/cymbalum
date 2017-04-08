import subprocess
import json
import os
import re

import click
import cpuinfo

BASE_REPO_URL = 'https://github.com/lumol-org/lumol.git'

INDIVIDUAL_BENCH_TEMPLATE = """
<details><summary>%s %s</summary>
  <p>\n\n
\n\n
```bash\n
%s\n\n
```\n\n

</p></details>
"""

COMPARISON_TEMPLATE = """
<details><summary>%s %s</summary>
  <p>\n\n
\n\n
```bash\n
%s\n\n
```\n\n

</p></details>
"""


def request_api(endpoint, data=None):
    url = 'https://api.github.com/repos/lumol-org/lumol' + endpoint
    username = os.environ['LUMOL_GH_USERNAME']
    token = os.environ['LUMOL_GH_TOKEN']

    cmd = "curl -sS %s  -u '%s:%s'" % (url, username, token)
    if data is not None:
        cmd += " --data '%s'" % json.dumps(data)

    out = subprocess.check_output(cmd, shell=True).decode('utf-8')
    return json.loads(out)


def get_master_commit():
    cmd = 'git log --oneline upstream/master -n 1'
    out = subprocess.check_output(cmd, shell=True).decode('utf-8')
    h, _, title = out.rstrip().partition(' ')
    return h, title


def get_commit_descriptions(n_commits):
    """
    Get hash and title of the `n_commits` latest commits on the branch.

    Also adds the commit at the HEAD of master in the end. If this
    commit is met earlier, stops at this commit (the master commit
    is guaranteed to be at the end of the result.

    :param n_commits:
    :return:
    """
    cmd = 'git log --oneline | head -n %s' % n_commits
    out = subprocess.check_output(cmd, shell=True).decode('utf-8')
    master_h, master_title = get_master_commit()

    descriptions = []
    for line in out.split('\n'):
        if line.rstrip() == '':
            continue
        h, _, title = line.partition(' ')
        descriptions.append((h, title))
        if h == master_h:
            break

    if descriptions[-1][0] != master_h:
        descriptions.append((master_h, master_title))

    return descriptions


def setup_cloned_repo(pr_id):
    response = request_api('/pulls/%s' % pr_id)
    clone_url = response['head']['repo']['clone_url']
    commit_id = response['head']['sha']

    subprocess.call('git clone %s cloned_repo' % clone_url, shell=True)
    os.chdir('cloned_repo')
    subprocess.call('git checkout %s' % commit_id, shell=True)
    subprocess.call('git remote add upstream %s' % BASE_REPO_URL, shell=True)
    subprocess.call('git fetch upstream master', shell=True)


class Benchmarker:
    def __init__(self, n_commits, output_dir):
        self.commit_descriptions = get_commit_descriptions(n_commits)
        self.output_dir = os.path.abspath(os.path.expanduser(output_dir))

    def run_warmup(self):
        print('=================== Warming up ==============================')
        for _ in range(3):
            subprocess.check_output('cargo bench', shell=True)

    def run_bench(self, h):
        cmd = 'cargo bench > %s/%s.txt' % (self.output_dir, h)
        subprocess.call(cmd, shell=True)

    def run_all_benches(self):
        for h, title in self.commit_descriptions:
            print('=================== Benching commit %s ==============================' % h)
            subprocess.call('git checkout %s' % h, shell=True)
            self.run_bench(h)
            print('=================== Done ==============================')

    def compare_benches(self):
        comparisons = {}
        master_h, _ = self.commit_descriptions[-1]
        for h, title in self.commit_descriptions[:-1]:
            cmd = 'cargo benchcmp %s/%s.txt %s/%s.txt --threshold 2 --variance' % (
                self.output_dir, master_h, self.output_dir, h
            )
            out = subprocess.check_output(cmd, shell=True).decode('utf-8')
            comparisons[h] = out

        return comparisons

    def comment_pr(self, pr_id):
        # Comparison benchmarks
        master_h, master_title = self.commit_descriptions[-1]
        comment = '## Comparing to master (%s)\nusing `--threshold 2, latest commit first`' % master_h

        comparisons = self.compare_benches()
        for h, title in self.commit_descriptions[:-1]:
            compare = comparisons[h]
            comment += COMPARISON_TEMPLATE % (h, title, compare)

        # Individual benchmarks
        comment += '\n## Individual benchmarks\n'

        for k, (h, title) in enumerate(self.commit_descriptions):
            with open('%s/%s.txt' % (self.output_dir, h)) as f:
                bench = f.read()
            comment += INDIVIDUAL_BENCH_TEMPLATE % (h, title, bench)

        info = cpuinfo.get_cpu_info()
        if info is not None:
            comment += '\n<br>**CPU**: %s' % info['brand']

        # Emit the request
        data = {
            'body': comment
        }
        request_api('/issues/%s/comments' % pr_id, data)


@click.command()
@click.argument('output_dir')
@click.argument('n_commits', type=click.INT)
@click.argument('pr_id', type=click.INT)
def main(output_dir, n_commits, pr_id):
    """
    Run the benchmarks for multiple commits on a PR and compare to master.

    The benchmark results are saved in OUTPUT_DIR, and a comment
    with a summary will be automatically added to the PR.
    This script requires the environment variables LUMOL_GH_USERNAME
    and LUMOL_GH_TOKEN to contain respectively the Github username
    and a personal access token.
    """

    setup_cloned_repo(pr_id)

    benchmarker = Benchmarker(n_commits, output_dir)
    benchmarker.run_warmup()
    benchmarker.run_all_benches()

    benchmarker.comment_pr(pr_id)
    os.chdir('..')
    subprocess.call('rm -rf cloned_repo', shell=True)


if __name__ == '__main__':
    main()