"""验证 Arch 包的路径、注册信息和提取后的真实 Fcitx 插件行为。"""
from pathlib import Path
import subprocess
import sys
import tempfile

ROOT = Path(__file__).resolve().parents[3]
package = Path(sys.argv[1]).resolve()
subprocess.run(["pacman", "-Qip", str(package)], check=True)
with tempfile.TemporaryDirectory(prefix="qingjian-package-test-") as temp:
    root = Path(temp)
    subprocess.run(["bsdtar", "--no-same-owner", "-xf", str(package), "-C", str(root)], check=True)
    files = ["usr/lib/fcitx5/qingjian.so", "usr/lib/fcitx5/libqingjian_linux.so",
             "usr/share/fcitx5/addon/qingjian.conf", "usr/share/fcitx5/inputmethod/qingjian.conf",
             "usr/share/qingjian/config.toml", "usr/share/qingjian/data/generated/dict.qj"]
    for name in files:
        if not (root / name).is_file():
            raise SystemExit(f"包中缺少 {name}")
    if (root / "home").exists():
        raise SystemExit("包中不应含任何用户 home 文件")
    text = (root / "usr/share/fcitx5/addon/qingjian.conf").read_text()
    if "Library=qingjian\n" not in text:
        raise SystemExit("系统安装包不得引用构建机用户目录")
    subprocess.run(["bash", "apps/linux/scripts/test.sh", "--prefix", str(root / "usr")], cwd=ROOT, check=True)
print("Arch 包验证通过：包元数据、路径、注册、资源与真实插件输入测试。")
