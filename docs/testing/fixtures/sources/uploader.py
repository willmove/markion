"""Local command-uploader fixture; never contacts a cloud service."""
import argparse
import shutil
import sys
import time
from pathlib import Path

parser = argparse.ArgumentParser()
parser.add_argument("--mode", choices=["ok", "fail", "multi", "slow"], default="ok")
parser.add_argument("file", type=Path)
args = parser.parse_args()
if args.mode == "fail":
    print("Synthetic uploader failure", file=sys.stderr)
    sys.exit(3)
if args.mode == "slow":
    time.sleep(10)
destination = Path(__file__).parent / "images" / "uploaded.png"
shutil.copyfile(args.file, destination)
print("http://127.0.0.1:18115/images/uploaded.png")
if args.mode == "multi":
    print("http://127.0.0.1:18115/images/checker.png")
