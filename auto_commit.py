import os
from subprocess import check_call

url=f'https://{os.environ['GITHUB_ACTOR']}:{os.environ['GITHUB_TOKEN']}@github.com/{os.environ['GITHUB_REPOSITORY']}.git'
check_call(['git', 'config', '--local', 'user.name', 'GitHub'])
check_call(['git', 'config', '--local', 'user.email', 'noreply@github.com'])
message=f'[auto-verifier] verify commit {os.environ['GITHUB_SHA']}'
check_call(['git', 'commit', '-m', message, '-a'])
check_call(['git', 'push', url, 'HEAD'])
