"""从已提交的源码构建 Arch 安装包，并保存对应源码与离线数据。"""
from __future__ import annotations

import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile
import tomllib

ROOT = Path(__file__).resolve().parents[3]


def run(*args: str, cwd: Path = ROOT, capture: bool = False, env=None):
    return subprocess.run(args, cwd=cwd, check=True, env=env,
                          stdout=subprocess.PIPE if capture else None,
                          text=capture).stdout


def sha(path: Path) -> str:
    with path.open("rb") as stream:
        return hashlib.file_digest(stream, "sha256").hexdigest()


def copy(source: Path, destination: Path):
    destination.parent.mkdir(parents=True, exist_ok=True)
    shutil.copy2(source, destination)


def main():
    if len(sys.argv) > 1:
        raise SystemExit("用法：apps/linux/scripts/package.sh（须先提交源码改动）")
    if run("git", "status", "--porcelain", capture=True).strip():
        raise SystemExit("请先提交源码改动，以便安装包与可下载源码严格对应。")
    for command in ["cargo", "c++", "pkg-config", "makepkg", "bsdtar", "zstd"]:
        if not shutil.which(command):
            raise SystemExit(f"缺少构建工具：{command}")
    if os.uname().machine != "x86_64":
        raise SystemExit("当前发布流程只验证过 x86_64。")
    version = tomllib.loads((ROOT / "apps/linux/Cargo.toml").read_text())["package"]["version"]
    if version.endswith("-dev"):
        raise SystemExit("发布前需将 apps/linux/Cargo.toml 的版本改为不带 -dev 的版本并提交。")
    commit = run("git", "rev-parse", "HEAD", capture=True).strip()
    epoch = run("git", "show", "-s", "--format=%ct", "HEAD", capture=True).strip()
    data = ROOT / "data/generated"
    if not (data / "dict.qj").is_file():
        run("bash", "apps/linux/scripts/fetch-data.sh")
    required = [data / name for name in ["dict.qj", "lm.qj", "glossary-en.qj", "glossary-ja.qj"]]
    required += sorted(p for p in (data / "dicts").glob("*.qj") if not p.name.startswith("."))
    if any(not p.is_file() for p in required) or not (data / "SOURCE-SHA256SUMS").is_file():
        raise SystemExit("缺少正式数据或校验记录，请运行 fetch-data.sh。")
    run("bash", "apps/linux/scripts/build.sh")
    run("bash", "apps/linux/scripts/test.sh")
    target = Path(os.environ.get("CARGO_TARGET_DIR", ROOT / "target"))
    if not target.is_absolute():
        target = ROOT / target
    out = ROOT / "dist" / f"linux-v{version}"
    out.mkdir(parents=True, exist_ok=True)
    fcitx = run("pkg-config", "--modversion", "Fcitx5Core", capture=True).strip()
    info = {
        "version": version, "commit": commit, "architecture": "x86_64",
        "source": f"https://github.com/marsjane/qingjian/tree/{commit}",
        "rustc": run("rustc", "--version", capture=True).strip(),
        "fcitx5": fcitx,
        "cxx": run("c++", "--version", capture=True).splitlines()[0],
        "data_bundle": (data / "SOURCE-SHA256SUMS").read_text().strip(),
        "data_files": {str(p.relative_to(data)): sha(p) for p in required},
    }
    (out / "build-info.json").write_text(json.dumps(info, indent=2, ensure_ascii=False) + "\n")
    source = out / f"qingjian-{version}-source.tar.gz"
    run("git", "archive", "--format=tar.gz", f"--prefix=qingjian-{version}/", f"--output={source}", "HEAD")
    # 数据文件单独发布；解到 data/generated 即可离线重建，无需依赖上游滚动 Release。
    data_archive = out / f"qingjian-{version}-data.tar.zst"
    run("bsdtar", "--zstd", "-cf", str(data_archive), "-C", str(data),
        *[str(p.relative_to(data)) for p in required], "SOURCE-SHA256SUMS")
    with tempfile.TemporaryDirectory(prefix="qingjian-arch-", dir=target) as temp:
        work = Path(temp)
        payload = work / "payload"
        share = payload / "usr/share/qingjian"
        for name in ["qingjian.so", "libqingjian_linux.so"]:
            copy(target / "linux" / name, payload / "usr/lib/fcitx5" / name)
        copy(ROOT / "apps/linux/fcitx5/addon.conf", payload / "usr/share/fcitx5/addon/qingjian.conf")
        copy(ROOT / "apps/linux/fcitx5/inputmethod.conf", payload / "usr/share/fcitx5/inputmethod/qingjian.conf")
        copy(ROOT / "apps/linux/config.toml", share / "config.toml")
        copy(ROOT / "apps/linux/scripts/remove-user-install.sh", payload / "usr/bin/qingjian-remove-user-install")
        for p in required:
            copy(p, share / "data/generated" / p.relative_to(data))
        copy(data / "SOURCE-SHA256SUMS", share / "data/generated/SOURCE-SHA256SUMS")
        copy(out / "build-info.json", share / "build-info.json")
        for p in ["apps/linux/README.md", "apps/linux/MAINTAINING.md", "apps/linux/VALIDATION.md"]:
            copy(ROOT / p, payload / "usr/share/doc/fcitx5-qingjian" / Path(p).name)
        for p in ["LICENSE", "docs/design/landscape.md", "assets/lexicon/README.md",
                  "assets/lexicon/00_meta/THUOCL_LICENSE.txt", "assets/emoji/LICENSE-unicode.txt",
                  "assets/glossary/README.md"]:
            copy(ROOT / p, payload / "usr/share/licenses/fcitx5-qingjian" / p)
        run("bsdtar", "--zstd", "-cf", str(work / "payload.tar.zst"), "-C", str(work), "payload")
        recipe = (ROOT / "apps/linux/packaging/PKGBUILD.in").read_text()
        recipe = recipe.replace("@VERSION@", version.replace("-", "."))
        recipe = recipe.replace("@FCITX_VERSION@", fcitx).replace("@PAYLOAD_SHA256@", sha(work / "payload.tar.zst"))
        (work / "PKGBUILD").write_text(recipe)
        copy(ROOT / "apps/linux/packaging/qingjian.install", work / "qingjian.install")
        env = os.environ.copy()
        env.update(PKGDEST=str(out), PACKAGER="marsjane <45665172+marsjane@users.noreply.github.com>", SOURCE_DATE_EPOCH=epoch)
        run("makepkg", "--force", "--noconfirm", cwd=work, env=env)
    packages = sorted(out.glob("fcitx5-qingjian-*.pkg.tar.zst"))
    if len(packages) != 1:
        raise SystemExit(f"安装包数量异常：{packages}")
    run("python3", "apps/linux/packaging/verify.py", str(packages[0]))
    files = [packages[0], source, data_archive, out / "build-info.json"]
    (out / "SHA256SUMS").write_text("".join(f"{sha(p)}  {p.name}\n" for p in files))
    print(f"已构建并验证安装包：{packages[0]}")


if __name__ == "__main__":
    main()
