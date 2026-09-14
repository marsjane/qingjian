#!/usr/bin/env bash
# 官方 data Release 是滚动发布；下载后校验并留下实际摘要。
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/../../.." && pwd)"
BASE=https://github.com/qingjian-team/qingjian/releases/download/data
mkdir -p "$ROOT/target" "$ROOT/data/generated"
TEMP="$(mktemp -d "$ROOT/target/data-download.XXXXXX")"
trap 'rm -rf "$TEMP"' EXIT
curl --fail --location --retry 3 "$BASE/SHA256SUMS" -o "$TEMP/SHA256SUMS"
curl --fail --location --retry 3 "$BASE/qingjian-data.tar.gz" -o "$TEMP/qingjian-data.tar.gz"
python3 - "$TEMP" "$ROOT/data/generated" <<'PYDATA'
import hashlib, pathlib, sys, tarfile
folder, output = map(pathlib.Path, sys.argv[1:])
name = 'qingjian-data.tar.gz'
checks = [line.split() for line in (folder/'SHA256SUMS').read_text().splitlines()]
expected = [h for h, p in checks if p.removeprefix('*').removeprefix('./') == name]
actual = hashlib.sha256((folder/name).read_bytes()).hexdigest()
if expected != [actual]:
    raise SystemExit('官方数据 SHA256 校验失败，未安装')
with tarfile.open(folder/name) as archive:
    archive.extractall(output, filter='data')
(output/'SOURCE-SHA256SUMS').write_text(f'{actual}  {name}\n')
print(f'词库校验通过：{actual}')
PYDATA
echo "已保存词库：$ROOT/data/generated"
