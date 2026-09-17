#!/usr/bin/env bash
# Configure git so commits are authored by willmove, without Cursor/Claude
# Co-authored-by trailers. Intended for Cursor Cloud / IDE / CLI agents.
set -euo pipefail

AUTHOR_NAME="willmove"
AUTHOR_EMAIL="willmove@qq.com"

usage() {
  cat <<'EOF'
Usage: scripts/configure-git-identity.sh [--self-test]

Sets this repository's local git user to willmove <willmove@qq.com>,
disables Cursor's commit-msg Co-authored-by injection when present, and
installs a hook that strips Cursor/Claude attribution trailers.
EOF
}

is_ai_attribution_line() {
  local line="$1"
  case "${line,,}" in
    co-authored-by:*cursor*|co-authored-by:*claude*|co-authored-by:*anthropic*|co-authored-by:*cursoragent@*|co-authored-by:*noreply@anthropic.com*)
      return 0
      ;;
    made-with:*cursor*|made\ with:*cursor*)
      return 0
      ;;
    *generated\ with*claude*|🤖*generated\ with*)
      return 0
      ;;
  esac
  return 1
}

strip_ai_attribution() {
  local msg_file="$1"
  local tmp
  tmp="$(mktemp)"
  while IFS= read -r line || [[ -n "$line" ]]; do
    if is_ai_attribution_line "$line"; then
      continue
    fi
    printf '%s\n' "$line" >> "$tmp"
  done < "$msg_file"
  # Drop trailing blank lines left behind after trailer removal.
  while [[ -s "$tmp" ]] && [[ "$(tail -n 1 "$tmp")" == "" ]]; do
    sed -i '$d' "$tmp"
  done
  mv "$tmp" "$msg_file"
}

write_strip_hook() {
  local dest="$1"
  cat > "$dest" <<'EOF'
#!/usr/bin/env bash
# Strip Cursor/Claude Co-authored-by and related AI attribution trailers.
set -euo pipefail
ROOT="$(git rev-parse --show-toplevel 2>/dev/null || true)"
if [[ -n "$ROOT" && -x "$ROOT/scripts/configure-git-identity.sh" ]]; then
  "$ROOT/scripts/configure-git-identity.sh" --strip-msg "$1"
  exit 0
fi
exit 0
EOF
  chmod +x "$dest"
}

disable_cursor_coauthor_hook() {
  local hooks_dir="$1"
  local hook="$hooks_dir/commit-msg.cursor.co-author"
  if [[ -e "$hook" || -L "$hook" ]]; then
    cat > "$hook" <<'EOF'
#!/bin/bash
# Disabled by scripts/configure-git-identity.sh:
# do not append Cursor/Claude Co-authored-by trailers.
exit 0
EOF
    chmod +x "$hook"
  fi
}

install_cursor_strip_hook() {
  local hooks_dir="$1"
  [[ -d "$hooks_dir" ]] || return 0
  disable_cursor_coauthor_hook "$hooks_dir"
  write_strip_hook "$hooks_dir/commit-msg.cursor.strip-ai-attribution"
  # When Cursor's dispatcher is absent, git only runs a file named commit-msg.
  if [[ ! -e "$hooks_dir/commit-msg" ]]; then
    write_strip_hook "$hooks_dir/commit-msg"
  fi
}

configure_repo() {
  local repo_root
  repo_root="$(git rev-parse --show-toplevel)"
  git -C "$repo_root" config --local user.name "$AUTHOR_NAME"
  git -C "$repo_root" config --local user.email "$AUTHOR_EMAIL"

  local hooks_path
  hooks_path="$(git -C "$repo_root" config --local --get core.hooksPath || true)"
  if [[ -n "$hooks_path" ]]; then
    if [[ "$hooks_path" != /* ]]; then
      hooks_path="$repo_root/$hooks_path"
    fi
    install_cursor_strip_hook "$hooks_path"
  fi

  # Also cover the original hooks directory Cursor chained from, when present.
  local original_ref="${hooks_path:-}/.cursor-original-hooks-path"
  if [[ -f "$original_ref" ]]; then
    local original
    original="$(cat "$original_ref")"
    if [[ -n "$original" && -d "$original" ]]; then
      write_strip_hook "$original/commit-msg"
    fi
  fi

  printf 'git identity: %s <%s>\n' \
    "$(git -C "$repo_root" config --get user.name)" \
    "$(git -C "$repo_root" config --get user.email)"
}

self_test() {
  local tmp
  tmp="$(mktemp -d)"
  trap "rm -rf $(printf '%q' "$tmp")" RETURN

  git init -q "$tmp"
  git -C "$tmp" config user.name "Cursor Agent"
  git -C "$tmp" config user.email "cursoragent@cursor.com"
  git -C "$tmp" config commit.gpgsign false

  mkdir -p "$tmp/scripts" "$tmp/hooks"
  cp "$0" "$tmp/scripts/configure-git-identity.sh"
  chmod +x "$tmp/scripts/configure-git-identity.sh"

  cat > "$tmp/hooks/.dispatcher" <<'EOF'
#!/bin/bash
set -e
HOOKS_DIR="$(cd "$(dirname "$0")" && pwd)"
HOOK_NAME="$(basename "$0")"
for cursor_hook in "$HOOKS_DIR"/$HOOK_NAME.cursor*; do
    if [ -x "$cursor_hook" ]; then
        "$cursor_hook" "$@"
    fi
done
EOF
  chmod +x "$tmp/hooks/.dispatcher"
  ln -s .dispatcher "$tmp/hooks/commit-msg"
  cat > "$tmp/hooks/commit-msg.cursor.co-author" <<'EOF'
#!/bin/bash
echo "" >> "$1"
echo "Co-authored-by: Cursor Agent <cursoragent@cursor.com>" >> "$1"
exit 0
EOF
  chmod +x "$tmp/hooks/commit-msg.cursor.co-author"
  git -C "$tmp" config core.hooksPath "$tmp/hooks"

  (cd "$tmp" && ./scripts/configure-git-identity.sh)

  local name email
  name="$(git -C "$tmp" config --get user.name)"
  email="$(git -C "$tmp" config --get user.email)"
  [[ "$name" == "$AUTHOR_NAME" ]] || {
    echo "self-test: expected user.name $AUTHOR_NAME, got $name" >&2
    exit 1
  }
  [[ "$email" == "$AUTHOR_EMAIL" ]] || {
    echo "self-test: expected user.email $AUTHOR_EMAIL, got $email" >&2
    exit 1
  }

  printf 'test\n' > "$tmp/README"
  git -C "$tmp" add README
  git -C "$tmp" commit -q -m $'test identity\n\nCo-authored-by: Cursor Agent <cursoragent@cursor.com>\nCo-authored-by: Claude <noreply@anthropic.com>'

  local author body
  author="$(git -C "$tmp" log -1 --format='%an <%ae>')"
  body="$(git -C "$tmp" log -1 --format='%b')"
  [[ "$author" == "$AUTHOR_NAME <$AUTHOR_EMAIL>" ]] || {
    echo "self-test: expected author $AUTHOR_NAME <$AUTHOR_EMAIL>, got $author" >&2
    exit 1
  }
  if grep -qiE 'Co-authored-by:.*(Cursor|Claude)|cursoragent@|noreply@anthropic' <<<"$body"; then
    echo "self-test: AI Co-authored-by trailer was not stripped:" >&2
    printf '%s\n' "$body" >&2
    exit 1
  fi

  echo "self-test passed: Author=$author (no Cursor/Claude Co-authored-by)"
}

main() {
  case "${1:-}" in
    -h|--help)
      usage
      ;;
    --strip-msg)
      [[ -n "${2:-}" ]] || { echo "missing commit-msg path" >&2; exit 2; }
      strip_ai_attribution "$2"
      ;;
    --self-test)
      self_test
      ;;
    "")
      configure_repo
      ;;
    *)
      usage >&2
      exit 2
      ;;
  esac
}

main "$@"
