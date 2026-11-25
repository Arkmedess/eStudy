#!/usr/bin/env bash
set -euo pipefail

# Script padrão de validação completa e benchmark do eStudy.
# Uso:
#   bash scripts/bench_and_validate.sh          # Validação completa + RSS ao vivo (se houver display)
#   bash scripts/bench_and_validate.sh --ci     # Modo headless para CI (pula abertura de janela gráfica)

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

CI_MODE=false
if [[ "${1:-}" == "--ci" ]]; then
  CI_MODE=true
fi

MAX_IDLE_RSS_MIB=45

echo "=================================================================="
echo "1. Formatação e Análise Estática (Zero Warnings)"
echo "=================================================================="
cargo fmt --all -- --check
timeout 180 cargo clippy --workspace --all-targets -- -D warnings

echo ""
echo "=================================================================="
echo "2. Suíte Completa de Testes (Unitários, Integração e Doctests)"
echo "=================================================================="
timeout 180 cargo test --workspace --all-targets
timeout 180 cargo test --workspace --doc

echo ""
echo "=================================================================="
echo "3. Compilação Release e Tamanho dos Binários"
echo "=================================================================="
timeout 300 cargo build --release -p estudy-desktop -p estudy-preview
ls -lh target/release/estudy-desktop target/release/estudy-preview

echo ""
echo "=================================================================="
echo "4. Benchmark em Release (Operações de Domínio/SQLite)"
echo "=================================================================="
timeout 120 cargo test --release --workspace --no-fail-fast

if [[ "$CI_MODE" == "false" ]] && { [[ -n "${WAYLAND_DISPLAY:-}" ]] || [[ -n "${DISPLAY:-}" ]]; }; then
  echo ""
  echo "=================================================================="
  echo "5. Medição ao Vivo de RAM (RSS) e Logs Estruturados JSON"
  echo "=================================================================="
  SANDBOX_DIR="$(mktemp -d)"
  trap 'rm -rf "$SANDBOX_DIR"' EXIT
  mkdir -p "$SANDBOX_DIR/data" "$SANDBOX_DIR/cache"

  XDG_CACHE_HOME="$SANDBOX_DIR/cache" ./target/release/estudy-desktop --data-dir "$SANDBOX_DIR/data" >/dev/null 2>&1 &
  DESKTOP_PID=$!
  sleep 2

  if kill -0 "$DESKTOP_PID" 2>/dev/null; then
    RSS_KB="$(awk '/VmRSS:/ {print $2}' "/proc/$DESKTOP_PID/status")"
    HWM_KB="$(awk '/VmHWM:/ {print $2}' "/proc/$DESKTOP_PID/status")"
    THREADS="$(awk '/Threads:/ {print $2}' "/proc/$DESKTOP_PID/status")"
    CPU_TICKS="$(awk '{print $14 + $15}' "/proc/$DESKTOP_PID/stat")"
    CLK_TCK="$(getconf CLK_TCK 2>/dev/null || echo 100)"
    CPU_MS="$(awk -v t="$CPU_TICKS" -v hz="$CLK_TCK" 'BEGIN {printf "%.0f", (t / hz) * 1000}')"
    CPU_PCT="$(awk -v ms="$CPU_MS" 'BEGIN {printf "%.1f", (ms / 2000.0) * 100}')"
    kill -TERM "$DESKTOP_PID" 2>/dev/null || true
    wait "$DESKTOP_PID" 2>/dev/null || true

    RSS_MIB="$(awk -v kb="$RSS_KB" 'BEGIN {printf "%.2f", kb / 1024}')"
    HWM_MIB="$(awk -v kb="$HWM_KB" 'BEGIN {printf "%.2f", kb / 1024}')"
    echo "estudy-desktop -> VmRSS: ${RSS_MIB} MiB (${RSS_KB} kB) | VmHWM: ${HWM_MIB} MiB | CPU(2s): ${CPU_MS} ms (~${CPU_PCT}%) | Threads: ${THREADS}"

    if awk -v rss="$RSS_MIB" -v max="$MAX_IDLE_RSS_MIB" 'BEGIN {exit !(rss > max)}'; then
      echo "ERRO: RSS (${RSS_MIB} MiB) excedeu o teto de ${MAX_IDLE_RSS_MIB} MiB."
      exit 1
    fi
  else
    echo "ERRO: estudy-desktop encerrou prematuramente."
    exit 1
  fi

  LOG_FILES=("$SANDBOX_DIR/cache/estudy/logs"/estudy.log*)
  if [[ -e "${LOG_FILES[0]}" ]]; then
    echo "Log estruturado gerado em: ${LOG_FILES[0]}"
    head -n 3 "${LOG_FILES[0]}"
  else
    echo "AVISO: arquivo de log JSON não encontrado em $SANDBOX_DIR/cache/estudy/logs"
  fi
else
  echo ""
  echo "5. Medição de janela gráfica pulada (modo --ci ou sem DISPLAY/WAYLAND_DISPLAY)."
fi

echo ""
echo "✅ Validação completa e benchmark aprovados!"
