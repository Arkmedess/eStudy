#!/usr/bin/env python3
"""Servidor MCP (stdio JSON-RPC) para capturar screenshots reais da UI do eStudy."""

from __future__ import annotations

import base64
import json
import os
import subprocess
import sys
import tempfile
from pathlib import Path
from typing import Any

REPO_ROOT = Path(__file__).resolve().parent.parent
DEBUG_BINARY = REPO_ROOT / "target" / "debug" / "estudy-desktop"

ROUTES = [
    "home",
    "projects",
    "habits",
    "second-brain",
    "calendar",
    "studies",
    "bookshelf",
    "finance",
    "focus",
    "garden",
    "shop",
    "settings",
]
THEMES = ["oliva", "menta", "cyber", "amber", "sakura"]
MODES = ["light", "dark"]


def _send_response(payload: dict[str, Any]) -> None:
    sys.stdout.write(json.dumps(payload, ensure_ascii=False) + "\n")
    sys.stdout.flush()


def _error_response(req_id: Any, code: int, message: str) -> dict[str, Any]:
    return {
        "jsonrpc": "2.0",
        "id": req_id,
        "error": {"code": code, "message": message},
    }


def _handle_capture(args: dict[str, Any]) -> dict[str, Any]:
    route = str(args.get("route", "home"))
    theme = str(args.get("theme", "oliva"))
    mode = str(args.get("mode", "light"))
    width = int(args.get("width", 1360))
    height = int(args.get("height", 840))
    sidebar = str(args.get("sidebar", "closed"))
    subview_raw = args.get("subview")
    subview = str(subview_raw).strip().lower() if isinstance(subview_raw, str) and subview_raw.strip() else None
    rebuild = bool(args.get("rebuild", False))

    if route not in ROUTES:
        return {
            "isError": True,
            "content": [{"type": "text", "text": f"Rota inválida '{route}'. Opções: {', '.join(ROUTES)}"}],
        }
    if theme not in THEMES:
        return {
            "isError": True,
            "content": [{"type": "text", "text": f"Tema inválido '{theme}'. Opções: {', '.join(THEMES)}"}],
        }
    if mode not in MODES:
        return {
            "isError": True,
            "content": [{"type": "text", "text": f"Modo inválido '{mode}'. Opções: {', '.join(MODES)}"}],
        }

    out_dir = Path(tempfile.gettempdir()) / "estudy-mcp-screenshots"
    out_dir.mkdir(parents=True, exist_ok=True)
    subview_suffix = f"_{subview}" if subview else ""
    out_path = out_dir / f"{route}{subview_suffix}_{theme}_{mode}_{width}x{height}.png"

    cli_args = [
        "--screenshot",
        str(out_path),
        "--route",
        route,
        "--theme",
        theme,
        "--mode",
        mode,
        "--size",
        f"{width}x{height}",
        "--sidebar",
        sidebar,
    ]
    if subview:
        cli_args.extend(["--subview", subview])

    if DEBUG_BINARY.exists() and not rebuild:
        cmd = [str(DEBUG_BINARY), *cli_args]
    else:
        cmd = ["cargo", "run", "-p", "estudy-desktop", "--", *cli_args]

    env = os.environ.copy()
    proc = subprocess.run(
        cmd,
        cwd=str(REPO_ROOT),
        env=env,
        capture_output=True,
        text=True,
        timeout=120,
        check=False,
    )

    if proc.returncode != 0 or not out_path.exists():
        err_msg = proc.stderr.strip() or proc.stdout.strip() or f"exit code {proc.returncode}"
        return {
            "isError": True,
            "content": [{"type": "text", "text": f"Falha ao capturar screenshot: {err_msg}"}],
        }

    png_bytes = out_path.read_bytes()
    b64_data = base64.b64encode(png_bytes).decode("ascii")
    return {
        "content": [
            {
                "type": "text",
                "text": (
                    f"Screenshot capturado: {out_path} "
                    f"(route={route}, theme={theme}, mode={mode}, size={width}x{height}, sidebar={sidebar}"
                    f"{f', subview={subview}' if subview else ''})"
                ),
            },
            {
                "type": "image",
                "data": b64_data,
                "mimeType": "image/png",
            },
        ]
    }


def _handle_tools_list() -> dict[str, Any]:
    return {
        "tools": [
            {
                "name": "capture_ui_screenshot",
                "description": (
                    "Renderiza uma tela real do eStudy (iced) e retorna o PNG para inspeção visual."
                ),
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "route": {"type": "string", "enum": ROUTES, "default": "home"},
                        "theme": {"type": "string", "enum": THEMES, "default": "oliva"},
                        "mode": {"type": "string", "enum": MODES, "default": "light"},
                        "width": {"type": "integer", "default": 1360, "minimum": 320},
                        "height": {"type": "integer", "default": 840, "minimum": 240},
                        "sidebar": {"type": "string", "enum": ["open", "closed"], "default": "closed"},
                        "subview": {
                            "type": "string",
                            "description": (
                                "Subvista ou estado opcional (week, calendar, kanban, list, task-modal, "
                                "day-peek, note-modal, today-drawer, palette, focus-running)."
                            ),
                        },
                        "rebuild": {"type": "boolean", "default": False},
                    },
                },
            },
            {
                "name": "list_ui_options",
                "description": "Lista rotas, temas e modos disponíveis para captura de screenshot.",
                "inputSchema": {"type": "object", "properties": {}},
            },
        ]
    }


def main() -> int:
    for raw_line in sys.stdin:
        line = raw_line.strip()
        if not line:
            continue
        try:
            req = json.loads(line)
        except json.JSONDecodeError as exc:
            _send_response(_error_response(None, -32700, f"JSON inválido: {exc}"))
            continue

        req_id = req.get("id")
        method = req.get("method", "")
        params = req.get("params") or {}

        if method == "notifications/initialized":
            continue

        if method == "initialize":
            _send_response(
                {
                    "jsonrpc": "2.0",
                    "id": req_id,
                    "result": {
                        "protocolVersion": "2024-11-05",
                        "capabilities": {"tools": {}},
                        "serverInfo": {"name": "estudy-ui", "version": "0.1.0"},
                    },
                }
            )
        elif method == "tools/list":
            _send_response({"jsonrpc": "2.0", "id": req_id, "result": _handle_tools_list()})
        elif method == "tools/call":
            name = params.get("name")
            args = params.get("arguments") or {}
            if name == "capture_ui_screenshot":
                result = _handle_capture(args)
            elif name == "list_ui_options":
                result = {
                    "content": [
                        {
                            "type": "text",
                            "text": json.dumps(
                                {"routes": ROUTES, "themes": THEMES, "modes": MODES},
                                ensure_ascii=False,
                            ),
                        }
                    ]
                }
            else:
                _send_response(_error_response(req_id, -32601, f"Tool desconhecida: {name}"))
                continue
            _send_response({"jsonrpc": "2.0", "id": req_id, "result": result})
        else:
            if req_id is not None:
                _send_response(_error_response(req_id, -32601, f"Método não suportado: {method}"))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
