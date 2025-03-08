#!/bin/bash

# エラーが発生したら即座に終了
set -e

echo "=== ローカルCIチェックを開始 ==="

# 変更されたファイルを検出
echo "変更されたファイルを検出中..."
CHANGED_FILES=$(git diff --name-only HEAD)
RUST_FILES=$(echo "$CHANGED_FILES" | grep -E '\.(rs|toml)$' || true)

if [ -z "$RUST_FILES" ]; then
    echo "Rustファイルの変更なし"
    exit 0
fi

# 変更のあったパッケージを特定
echo "変更のあったパッケージを特定中..."
PACKAGES=""
echo "$RUST_FILES" | while read -r file; do
    if [[ $file == engine/* ]]; then
        PACKAGES="$PACKAGES engine"
    elif [[ $file == game/* ]]; then
        PACKAGES="$PACKAGES game"
    elif [[ $file == model/* ]]; then
        PACKAGES="$PACKAGES model"
    elif [[ $file == data-editor/* ]]; then
        PACKAGES="$PACKAGES data-editor"
    fi
done

# 重複を除去
PACKAGES=$(echo "$PACKAGES" | tr ' ' '\n' | sort -u | tr '\n' ' ')

echo "ビルドチェックを実行中..."
cargo check

# フォーマットチェック
echo "フォーマットをチェック中..."
cargo fmt -- --check

# Clippyチェック
if [ -n "$PACKAGES" ]; then
    echo "Clippyチェックを実行中..."
    for package in $PACKAGES; do
        echo "パッケージ $package をチェック中..."
        cargo clippy -p $package -- -D warnings
    done

    # テストを実行
    echo "テストを実行中..."
    for package in $PACKAGES; do
        echo "パッケージ $package のテストを実行中..."
        cargo test -p $package
    done
else
    # パッケージが特定できない場合はワークスペース全体をチェック
    echo "ワークスペース全体をチェック中..."
    cargo clippy --all-targets -- -D warnings
    cargo test --all
fi

echo "=== すべてのチェックが完了しました ==="