#!/usr/bin/env bash
# 发布规则：推送后按 Cargo.toml 最新版本，增量创建 git tag + GitHub Release。
#
# 用法:
#   ./scripts/release.sh          # 正常发布：push → tag vX.Y.Z → gh release
#   ./scripts/release.sh --dry-run # 只打印将要执行的命令，不实际执行
#
# 设计：
#   - 版本单一事实来源 = Cargo.toml [package].version（与 README/Cargo 同步）
#   - 增量：tag / release 已存在则跳过，幂等可重复执行
#   - 发布入口人工触发（如需自动触发可再挂 GitHub Actions on:push）

set -euo pipefail
cd "$(dirname "$0")/.."

# ---------- 1. 获取最新版本 ----------
VERSION="$(sed -n '/^\[package\]/,/^\[/p' Cargo.toml | grep -m1 '^version' | cut -d'"' -f2)"
[[ -n "$VERSION" ]] || { echo "错误: 未能从 Cargo.toml [package] 解析版本" >&2; exit 1; }

DRY=false
TAG_OVERRIDE=""
while [[ $# -gt 0 ]]; do
  case "$1" in
    --dry-run) DRY=true; shift ;;
    --tag) TAG_OVERRIDE="$2"; shift 2 ;;
    *) echo "未知参数: $1" >&2; exit 1 ;;
  esac
done
TAG="${TAG_OVERRIDE:-v${VERSION}}"
[[ "$TAG" =~ ^v[0-9]+\.[0-9]+\.[0-9]+$ ]] || { echo "错误: tag 格式非法（须 vMAJOR.MINOR.PATCH）" >&2; exit 1; }
echo "==> 最新版本: ${VERSION}  (tag: ${TAG})"
run() { echo "\$ $*"; $DRY || "$@"; }

# ---------- 2. 获取远端 ----------
REMOTE="$(git remote | head -1)"
[[ -n "$REMOTE" ]] || { echo "错误: 未配置 git remote" >&2; exit 1; }

# ---------- 3. 推送当前分支 ----------
if ! git rev-parse --verify HEAD >/dev/null 2>&1; then
  echo "错误: 当前分支尚无任何提交，请先 commit 再运行发布" >&2
  exit 1
fi
run git push "$REMOTE" HEAD

# ---------- 4. 增量创建 git tag ----------
if git rev-parse "$TAG" >/dev/null 2>&1; then
  echo "==> tag ${TAG} 已存在，跳过"
else
  run git tag "$TAG"
  run git push "$REMOTE" "$TAG"
fi

# ---------- 5. 增量创建 GitHub Release ----------
if command -v gh >/dev/null 2>&1 && gh release view "$TAG" >/dev/null 2>&1; then
  echo "==> release ${TAG} 已存在，跳过"
else
  # Release notes：取自该 tag 之前的提交记录（无先前 tag 则从首个提交起）
  PREV="$(git tag --sort=-version:refname | head -1 || true)"
  [[ -z "$PREV" || "$PREV" == "$TAG" ]] && PREV="$(git rev-list --max-parents=0 HEAD)"
  NOTES="$(git log --oneline "${PREV}..HEAD" | sed 's/^[ \t]*//' | sed '/^$/d' | head -30 || true)"
  [[ -z "$NOTES" ]] && NOTES="发布 ${TAG}"
  run gh release create "$TAG" \
    --title "insurance-service ${TAG}" \
    --notes "$(printf '保险服务平台后端 %s\n\n%s' "${TAG}" "$NOTES")"
fi

echo "==> 完成: ${TAG} 已发布（本地 tag + GitHub release）"