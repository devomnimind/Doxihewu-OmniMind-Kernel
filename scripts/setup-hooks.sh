#!/bin/bash
# scripts/setup-hooks.sh — Instala git hooks localmente
# Os hooks em .git/hooks/ não são versionados pelo git.
# Este script copia os hooks versionados de scripts/hooks/ para .git/hooks/

set -e

HOOKS_DIR=".git/hooks"
SOURCE_DIR="scripts/hooks"

if [ ! -d "$HOOKS_DIR" ]; then
  echo "❌ .git/hooks/ não encontrado. Execute na raiz do repositório."
  exit 1
fi

for hook in post-commit pre-push; do
  if [ -f "$SOURCE_DIR/$hook" ]; then
    cp "$SOURCE_DIR/$hook" "$HOOKS_DIR/$hook"
    chmod +x "$HOOKS_DIR/$hook"
    echo "✅ $hook instalado"
  fi
done

echo ""
echo "Hooks instalados! Os hooks rodam automaticamente:"
echo "  post-commit → clippy + semgrep + cargo audit (feedback local)"
echo "  pre-push    → clippy -D warnings + cargo audit --deny (bloqueia push com erros)"
echo ""
echo "Para remover: rm .git/hooks/post-commit .git/hooks/pre-push"
