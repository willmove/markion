"""Prepare isolated synthetic acceptance material; no UI or network automation.

Python 3.10+, standard library only. Existing run directories are never replaced.
"""
from __future__ import annotations

import argparse
import base64
import csv
import hashlib
import io
import json
import os
from pathlib import Path
import re
import shutil
import struct
import subprocess
import sys
from urllib.parse import unquote
import zipfile
import zlib

SUITE = Path(__file__).resolve().parents[1]
SOURCES = Path(__file__).resolve().parent / "sources"
CASE_PATTERN = re.compile(r"^## ([DERWXGAP]\d{2}) .+$", re.MULTILINE)


def digest(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def put(root: Path, name: str, data: bytes | str) -> None:
    path = root / name
    path.parent.mkdir(parents=True, exist_ok=True)
    with path.open("xb") as output:
        output.write(data.encode("utf-8") if isinstance(data, str) else data)


def png(extra_bytes: int = 0) -> bytes:
    def chunk(kind: bytes, data: bytes) -> bytes:
        return struct.pack(">I", len(data)) + kind + data + struct.pack(">I", zlib.crc32(kind + data))

    rows = bytearray()
    for y in range(128):
        rows.append(0)
        for x in range(128):
            rows.extend((24, 104, 170) if (x // 16 + y // 16) % 2 else (245, 212, 55))
    content = b"\x89PNG\r\n\x1a\n" + chunk(b"IHDR", struct.pack(">IIBBBBB", 128, 128, 8, 2, 0, 0, 0))
    if extra_bytes:
        content += chunk(b"tEXt", b"QA\x00" + b"X" * extra_bytes)
    return content + chunk(b"IDAT", zlib.compress(bytes(rows), 9)) + chunk(b"IEND", b"")


def synthetic_docx() -> bytes:
    parts = {
        "[Content_Types].xml": '<?xml version="1.0"?><Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types"><Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/><Default Extension="xml" ContentType="application/xml"/><Override PartName="/word/document.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml"/></Types>',
        "_rels/.rels": '<?xml version="1.0"?><Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"><Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="word/document.xml"/></Relationships>',
        "word/document.xml": '<?xml version="1.0"?><w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:body><w:p><w:r><w:t>Synthetic QA 中文😀</w:t></w:r></w:p><w:p><w:r><w:t xml:space="preserve">Alpha </w:t></w:r><w:r><w:rPr><w:b/></w:rPr><w:t>bold</w:t></w:r></w:p><w:tbl><w:tblGrid><w:gridCol w:w="2400"/><w:gridCol w:w="2400"/></w:tblGrid><w:tr><w:tc><w:p><w:r><w:t>A</w:t></w:r></w:p></w:tc><w:tc><w:p><w:r><w:t>B</w:t></w:r></w:p></w:tc></w:tr><w:tr><w:tc><w:p><w:r><w:t>1</w:t></w:r></w:p></w:tc><w:tc><w:p><w:r><w:t>中文</w:t></w:r></w:p></w:tc></w:tr></w:tbl><w:sectPr/></w:body></w:document>',
    }
    stream = io.BytesIO()
    with zipfile.ZipFile(stream, "w", compression=zipfile.ZIP_DEFLATED) as package:
        for name, content in parts.items():
            item = zipfile.ZipInfo(name, (2026, 1, 1, 0, 0, 0))
            item.compress_type = zipfile.ZIP_DEFLATED
            package.writestr(item, content.encode("utf-8"))
    return stream.getvalue()


def make_git(root: Path) -> None:
    if shutil.which("git") is None:
        raise ValueError("Git is unavailable; choose a NEW run with --without-git and mark Git cases BLOCKED")
    git_root = root / "git"
    git_root.mkdir()
    put(git_root, "empty.gitconfig", "")
    (git_root / "empty-hooks").mkdir()
    env = os.environ.copy()
    env.update({"GIT_CONFIG_GLOBAL": str(git_root / "empty.gitconfig"), "GIT_CONFIG_NOSYSTEM": "1", "GIT_TERMINAL_PROMPT": "0", "GIT_AUTHOR_DATE": "2026-01-01T00:00:00+0000", "GIT_COMMITTER_DATE": "2026-01-01T00:00:00+0000"})

    def git(*args: str) -> None:
        subprocess.run(["git", "-c", "init.templateDir=", "-c", "commit.gpgsign=false", "-c", "core.autocrlf=false", *args], env=env, check=True, capture_output=True, text=True)

    git("init", "--bare", "--initial-branch=main", str(git_root / "remote.git"))
    git("clone", str(git_root / "remote.git"), str(git_root / "local"))
    local = git_root / "local"
    for key, value in (("user.name", "Markion QA"), ("user.email", "qa@example.invalid"), ("commit.gpgsign", "false"), ("core.autocrlf", "false"), ("core.hooksPath", str(git_root / "empty-hooks"))):
        git("-C", str(local), "config", key, value)
    put(local, "note.md", "# Sync fixture\n\nBASE note\n")
    put(local, "other.md", "# Other fixture\n\nBASE other\n")
    put(local, "unrelated.md", "# Unrelated staging fixture\n\nBASE unrelated\n")
    git("-C", str(local), "add", "note.md", "other.md", "unrelated.md")
    git("-C", str(local), "commit", "-m", "QA baseline")
    git("-C", str(local), "push", "-u", "origin", "main")
    git("clone", str(git_root / "remote.git"), str(git_root / "peer"))
    for key, value in (("user.name", "Markion QA"), ("user.email", "qa@example.invalid"), ("commit.gpgsign", "false"), ("core.autocrlf", "false"), ("core.hooksPath", str(git_root / "empty-hooks"))):
        git("-C", str(git_root / "peer"), "config", key, value)


def prepare(output: Path, without_git: bool) -> None:
    root = output.resolve()
    repo = SUITE.parents[1]
    if root == repo or root in repo.parents or root.is_relative_to(SUITE):
        raise ValueError("Choose a dedicated run directory outside docs/testing and repository ancestors")
    root.mkdir(parents=True, exist_ok=False)
    workspace = root / "workspace"
    workspace.mkdir()
    for source in sorted(SOURCES.iterdir()):
        if source.is_symlink() or not source.is_file():
            raise ValueError(f"Unexpected fixture source: {source.name}")
        # Normalize source checkout line endings; explicit byte variants follow.
        put(workspace, source.name, source.read_text(encoding="utf-8"))
    text = (workspace / "01-editing.md").read_bytes()
    put(workspace, "line-endings/lf.md", text)
    put(workspace, "line-endings/crlf.md", text.replace(b"\n", b"\r\n"))
    put(workspace, "line-endings/no-eof.md", text.rstrip(b"\n"))
    for name in ("02-structures", "03-tables"):
        content = (workspace / (name + ".md")).read_bytes()
        put(workspace, "line-endings/" + name + "-crlf.md", content.replace(b"\n", b"\r\n"))
        put(workspace, "line-endings/" + name + "-no-eof.md", content.rstrip(b"\n"))
    put(workspace, "empty.md", "")
    put(workspace, "images/checker.png", png())
    put(workspace, "images/large.png", png(9 * 1024 * 1024))
    put(workspace, "images/vector.svg", '<svg xmlns="http://www.w3.org/2000/svg" width="128" height="128"><rect width="128" height="128" fill="#1868aa"/><circle cx="64" cy="64" r="40" fill="#f5d437"/></svg>\n')
    uri = base64.b64encode(png(2 * 1024 * 1024)).decode("ascii")
    with (workspace / "05-images.md").open("ab") as image_doc:
        image_doc.write(("\n![data-uri](data:image/png;base64," + uri + ")\n").encode())
    put(workspace, "data/notes.txt", "Plain **literal** 中文😀\n")
    put(workspace, "data/sample.json", '{"title":"中文😀","items":[1,2,3]}\n')
    put(workspace, "data/sample.yaml", 'title: "中文😀"\nitems: [1, 2, 3]\n')
    put(workspace, "data/sample.toml", 'title = "中文😀"\nitems = [1, 2, 3]\n')
    put(workspace, "ai/a.md", "Alpha 中文😀\n\nSecond paragraph A.\n")
    put(workspace, "ai/b.md", "Beta\n\nSecond paragraph B.\n")
    put(workspace, "imports/synthetic.docx", synthetic_docx())
    put(workspace, "imports/synthetic.expected.txt", "Synthetic QA 中文😀\nAlpha bold\nA\tB\n1\t中文\n")
    put(workspace, "imports/invalid.docx", b"Not a ZIP package\n")
    unit = ("## Large region\n\nhello 中文😀 **bold** and [label](https://example.com).\n\n- first\n- second\n\n| A | B |\n| --- | --- |\n| alpha | beta |\n\n```rust\nfn qa() {}\n```\n\n").encode()
    for kb in (100, 300, 600, 1024):
        size = kb * 1024
        put(workspace, f"large/mixed-{kb}kb.md", unit * (size // len(unit)) + b" " * (size % len(unit)))
    put(workspace, "large/dense-line.md", "# Dense line\n\n" + "needle " * 10000 + "中文😀\n")
    put(workspace, "tree/.hidden.md", "# Hidden fixture\n")
    put(workspace, "tree/hidden-attribute.md", "# Windows hidden-attribute fixture\n")
    for name in ("target", "node_modules", ".git"):
        put(workspace, f"tree/{name}/excluded.md", "# Must remain excluded\n")
    (workspace / "tree/empty").mkdir()
    put(workspace, "tree/deep/one/two/three/four/five/six/leaf.md", "# Deep fixture\n")
    for index in range(120):
        put(workspace, f"tree/large/{index:03d}-很长的中文文件名称-QA.md", f"# Tree fixture {index}\n")
    baseline = root / "baseline"
    shutil.copytree(workspace, baseline)
    config = "check_for_updates_on_startup = false\n[auto_save]\nenabled = false\n"
    put(root, "profile/roaming/Markion/config.toml", config)
    put(root, "profile/config/markion/config.toml", config)
    for name in ("profile/local", "profile/cache", "evidence/expected"):
        (root / name).mkdir(parents=True, exist_ok=True)
    for name in ("release-report.md", "results.csv", "docx-corpus.csv"):
        put(root, name, (SUITE / "templates" / name).read_text(encoding="utf-8"))
    if not without_git:
        make_git(root)
    files = [{"path": str(path.relative_to(workspace)).replace("\\", "/"), "bytes": path.stat().st_size, "sha256": digest(path.read_bytes())} for path in sorted(workspace.rglob("*")) if path.is_file()]
    manifest = {"schema": 1, "origin": "repository-owned synthetic acceptance fixtures", "git": "not-prepared" if without_git else "local-bare-remote-and-two-clones", "files": files}
    put(root, "fixture-manifest.json", json.dumps(manifest, ensure_ascii=False, indent=2) + "\n")
    print(json.dumps({"run": str(root), "files": len(files), "manifest_sha256": digest((root / "fixture-manifest.json").read_bytes()), "native_acceptance": "NOT_RUN"}))


def verify(root: Path, working: bool) -> None:
    manifest = json.loads((root / "fixture-manifest.json").read_text(encoding="utf-8"))
    for directory in (["baseline", "workspace"] if working else ["baseline"]):
        base = (root / directory).resolve()
        for entry in manifest["files"]:
            path = (base / entry["path"]).resolve()
            if not path.is_relative_to(base):
                raise ValueError("Manifest path escapes fixture directory")
            data = path.read_bytes()
            if len(data) != entry["bytes"] or digest(data) != entry["sha256"]:
                raise ValueError(f"Fixture identity mismatch: {directory}/{entry['path']}")
    print("Fixture identities verified; native acceptance remains NOT_RUN")


def compare(expected: Path, actual: Path) -> int:
    left, right = expected.read_bytes(), actual.read_bytes()
    equal = left == right
    first = next((i for i, (a, b) in enumerate(zip(left, right)) if a != b), min(len(left), len(right)))
    print(json.dumps({"equal": equal, "expected_bytes": len(left), "actual_bytes": len(right), "expected_sha256": digest(left), "actual_sha256": digest(right), "first_difference": None if equal else first}))
    return 0 if equal else 1


def check() -> None:
    cases = []
    for path in sorted((SUITE / "cases").glob("*.md")):
        text = path.read_text(encoding="utf-8")
        blocks = re.split(r"(?=^## [DERWXGAP]\d{2} )", text, flags=re.MULTILINE)[1:]
        for block in blocks:
            case_id = CASE_PATTERN.search(block).group(1)
            cases.append(case_id)
            for label in ("优先级", "前置与素材", "步骤", "预期", "变体（逐项建行）", "证据与收尾"):
                if f"**{label}：**" not in block:
                    raise ValueError(f"{case_id}: missing {label}")
    expected = {f"{prefix}{i:02}" for prefix, count in (("D", 10), ("E", 13), ("R", 8), ("W", 6), ("X", 8), ("G", 5), ("A", 5), ("P", 7)) for i in range(1, count + 1)}
    if set(cases) != expected or len(cases) != len(expected):
        raise ValueError("Case inventory missing or duplicated; update case and template contracts together")
    with (SUITE / "templates/results.csv").open(encoding="utf-8", newline="") as stream:
        rows = list(csv.DictReader(stream))
    if {row["case_id"] for row in rows} != expected or len(rows) != len(expected) or any(row["result"] != "NOT_RUN" for row in rows):
        raise ValueError("Result template must contain every case once, all NOT_RUN")
    # Deliberately broken images/links inside test documents are material, not docs.
    pages = [path for path in SUITE.rglob("*.md") if not path.is_relative_to(SOURCES)]
    pages.append(SUITE.parent / "release-process.md")
    for path in pages:
        for target in re.findall(r"\[[^\]\n]+\]\(([^)\n]+)\)", path.read_text(encoding="utf-8")):
            if "://" in target or target.startswith(("#", "mailto:")):
                continue
            destination = unquote(target.split("#", 1)[0]).strip("<>")
            if destination and not (path.parent / destination).exists():
                raise ValueError(f"Broken documentation link: {path.relative_to(SUITE.parent)} -> {target}")
    print(f"Checked {len(cases)} cases, blank result rows, and {len(pages)} documentation pages; no native tests run")


def main() -> int:
    # Stable UTF-8 diagnostics even when a Windows parent captures a legacy codepage.
    sys.stdout.reconfigure(encoding="utf-8")
    sys.stderr.reconfigure(encoding="utf-8")
    parser = argparse.ArgumentParser(description=__doc__)
    commands = parser.add_subparsers(dest="command", required=True)
    prep = commands.add_parser("prepare")
    prep.add_argument("--output", type=Path, required=True)
    prep.add_argument("--without-git", action="store_true")
    identity = commands.add_parser("verify")
    identity.add_argument("--run", type=Path, required=True)
    identity.add_argument("--working", action="store_true")
    diff = commands.add_parser("compare")
    diff.add_argument("--expected", type=Path, required=True)
    diff.add_argument("--actual", type=Path, required=True)
    commands.add_parser("check")
    args = parser.parse_args()
    try:
        if args.command == "prepare":
            prepare(args.output, args.without_git)
        elif args.command == "verify":
            verify(args.run.resolve(), args.working)
        elif args.command == "compare":
            return compare(args.expected, args.actual)
        else:
            check()
    except (OSError, ValueError, KeyError, subprocess.CalledProcessError, zipfile.BadZipFile) as error:
        print(f"ERROR: {error}. Existing/partial run directories are preserved; use a new run name.", file=sys.stderr)
        return 2
    return 0


if __name__ == "__main__":
    sys.exit(main())
