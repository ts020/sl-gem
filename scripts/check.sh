#!/bin/bash

# エラーが発生したら即座に終了
set -e

echo "=== ローカルCIチェックを開始 ==="

# 変更されたファイルを検出
echo "変更されたファイルを検出中..."
CHANGED_FILES=$(git diff --name-only HEAD)
RUST_FILES=$(echo "$CHANGED_FILES" | grep -E '\.(rs|toml|lock)$' || true)

if [ -z "$RUST_FILES" ]; then
    echo "Rustファイルの変更なし"
    exit 0
fi

# 変更のあったパッケージを特定
echo "変更のあったパッケージを特定中..."
CHANGED_PACKAGES=""
echo "$RUST_FILES" | while read -r file; do
    if [[ $file == engine/* ]]; then
        CHANGED_PACKAGES="$CHANGED_PACKAGES engine"
    elif [[ $file == game/* ]]; then
        CHANGED_PACKAGES="$CHANGED_PACKAGES game"
    elif [[ $file == model/* ]]; then
        CHANGED_PACKAGES="$CHANGED_PACKAGES model"
    elif [[ $file == data-editor/* ]]; then
        CHANGED_PACKAGES="$CHANGED_PACKAGES data-editor"
    fi
done

# 重複を除去
CHANGED_PACKAGES=$(echo "$CHANGED_PACKAGES" | tr ' ' '\n' | sort -u)

# 依存関係を解析するPythonスクリプトを作成
cat << 'EOF' > /tmp/analyze_deps.py
import json
import sys

def get_dependent_packages(metadata, changed_packages):
    # パッケージIDからパッケージ名へのマッピングを作成
    id_to_name = {pkg['id']: pkg['name'] for pkg in metadata['packages']}
    
    # パッケージ名から依存されているパッケージを見つける
    affected_packages = set(changed_packages)
    
    # 変更されたパッケージに依存する全てのパッケージを見つける
    for pkg in metadata['packages']:
        pkg_name = pkg['name']
        for dep in pkg.get('dependencies', []):
            dep_name = dep.get('name', '')
            if dep_name in changed_packages and pkg_name not in affected_packages:
                affected_packages.add(pkg_name)
    
    return affected_packages

metadata = json.loads(sys.stdin.read())
changed_packages = sys.argv[1].split(',')

affected_packages = get_dependent_packages(metadata, changed_packages)
print(' '.join(f'-p {pkg}' for pkg in affected_packages))
EOF

# 依存関係を解析して影響を受けるパッケージを特定
if [ -n "$CHANGED_PACKAGES" ]; then
    echo "依存関係を解析中..."
    PACKAGES_ARGS=$(cargo metadata --format-version=1 | python3 /tmp/analyze_deps.py "$(echo "$CHANGED_PACKAGES" | tr '\n' ',')")
else
    PACKAGES_ARGS=""
fi

# フォーマットチェック
echo "フォーマットをチェック中..."
cargo fmt -- --check

# テストとClippyチェックを実行
if [ -n "$PACKAGES_ARGS" ]; then
    echo "影響を受けるパッケージのテストを実行中..."
    cargo test $PACKAGES_ARGS --verbose
    
    echo "影響を受けるパッケージのClippyチェックを実行中..."
    cargo clippy $PACKAGES_ARGS -- -D warnings
else
    # パッケージが特定できない場合はワークスペース全体をチェック
    echo "ワークスペース全体をチェック中..."
    cargo test --all --verbose
    cargo clippy --all-targets -- -D warnings
fi

echo "=== すべてのチェックが完了しました ==="