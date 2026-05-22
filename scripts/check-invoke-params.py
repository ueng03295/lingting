#!/usr/bin/env python3
"""
Tauri Invoke Parameter Checker — 前端 invoke() 参数名 ↔ Rust 函数签名 自动比对

扫描前端所有 invoke() / invokeTauri() 调用，提取命令名和参数名，
与 Rust #[tauri::command] 函数签名逐项比对，报告 camelCase→snake_case 不匹配。

用法:
    python3 scripts/check-invoke-params.py [项目根目录]

退出码: 0=无问题, 1=发现不匹配
"""

import sys
import os
import re
import json
from pathlib import Path
from dataclasses import dataclass, field
from typing import Optional


@dataclass
class RustCommand:
    name: str
    params: list  # list of (name, type_str)
    has_rename_all_camel: bool
    file: str
    line: int


@dataclass
class FrontendInvoke:
    command: str
    params: dict  # {param_name: line_context}
    file: str
    line: int


@dataclass
class Mismatch:
    invoke: FrontendInvoke
    rust_cmd: RustCommand
    issues: list  # list of {frontend_param, rust_param, issue_type}


def find_project_root(start_path: str) -> str:
    """Find project root by locating frontend/src-tauri/"""
    start = Path(start_path).resolve()
    for parent in [start] + list(start.parents):
        if (parent / "frontend" / "src-tauri").is_dir():
            return str(parent)
    raise SystemExit(f"ERROR: Cannot find project root (no frontend/src-tauri/ under {start_path})")


def parse_rust_commands(project_root: str) -> dict:
    """Parse all #[tauri::command] functions and their parameter names."""
    commands = {}
    tauri_dir = os.path.join(project_root, "frontend", "src-tauri", "src")

    for root, dirs, files in os.walk(tauri_dir):
        # Skip target directory
        dirs[:] = [d for d in dirs if d != "target"]
        for fname in files:
            if not fname.endswith(".rs"):
                continue
            fpath = os.path.join(root, fname)
            rel_path = os.path.relpath(fpath, project_root)

            try:
                with open(fpath, "r", encoding="utf-8") as f:
                    content = f.read()
                    lines = content.split("\n")
            except Exception:
                continue

            # Find all #[tauri::command] annotations
            for i, line in enumerate(lines):
                if "#[tauri::command]" in line or "#[tauri::command(" in line:
                    # Find the function definition (may be on same line or next lines)
                    func_start = i + 1
                    func_line = ""
                    for j in range(i + 1, min(i + 10, len(lines))):
                        if "pub async fn" in lines[j] or "pub fn" in lines[j]:
                            func_line = lines[j]
                            func_start = j
                            break

                    if not func_line:
                        continue

                    # Extract function name
                    match = re.search(r"(?:pub\s+)?(?:async\s+)?fn\s+(\w+)\s*\(", func_line)
                    if not match:
                        continue

                    cmd_name = match.group(1)

                    # Extract parameters
                    # Get everything between ( and ) — may span multiple lines
                    full_sig = func_line
                    paren_count = full_sig.count("(") - full_sig.count(")")
                    if paren_count > 0:
                        for j in range(func_start + 1, min(func_start + 20, len(lines))):
                            full_sig += "\n" + lines[j]
                            paren_count += lines[j].count("(") - lines[j].count(")")
                            if paren_count <= 0:
                                break

                    # Parse parameters — extract names before ':' that aren't self/app/state
                    param_names = []
                    # Match patterns like: param_name: Type
                    for m in re.finditer(r"(\w+)\s*:\s*(?!R:|State|AppHandle)", full_sig.split("(")[1].split(")")[0] if "(" in full_sig else ""):
                        pname = m.group(1)
                        # Skip Rust special types
                        if pname in ("self", "app", "state", "config") and m.group(0).startswith(("app:", "state:", "config:")):
                            continue
                        # Skip type annotations in the type part
                        if pname in ("Runtime", "Wry", "R"):
                            continue
                        param_names.append(pname)

                    # Check if the function or its struct has rename_all = "camelCase"
                    has_rename = False
                    # Check nearby lines for serde(rename_all = "camelCase")
                    context_start = max(0, func_start - 20)
                    context_lines = "\n".join(lines[context_start:func_start + 1])
                    if 'rename_all' in context_lines and 'camelCase' in context_lines:
                        has_rename = True

                    commands[cmd_name] = RustCommand(
                        name=cmd_name,
                        params=param_names,
                        has_rename_all_camel=has_rename,
                        file=rel_path,
                        line=func_start + 1,
                    )

    return commands


def parse_frontend_invokes(project_root: str) -> list:
    """Parse all invoke() / invokeTauri() calls in frontend source."""
    invokes = []
    src_dir = os.path.join(project_root, "frontend", "src")

    for root, dirs, files in os.walk(src_dir):
        dirs[:] = [d for d in dirs if d not in ("node_modules", ".next", "out")]
        for fname in files:
            if not (fname.endswith(".tsx") or fname.endswith(".ts")):
                continue
            fpath = os.path.join(root, fname)
            rel_path = os.path.relpath(fpath, project_root)

            try:
                with open(fpath, "r", encoding="utf-8") as f:
                    content = f.read()
            except Exception:
                continue

            # Match invoke('command_name', { ... }) or invokeTauri('command_name', { ... })
            # Handle multiline invocations
            pattern = r"(?:invoke|invokeTauri)\s*\(\s*['\"](\w+)['\"]\s*,\s*\{([^}]*)\}"

            for m in re.finditer(pattern, content):
                cmd_name = m.group(1)
                params_str = m.group(2)
                line_num = content[:m.start()].count("\n") + 1

                # Extract parameter names
                params = {}
                for pm in re.finditer(r"(\w+)\s*:", params_str):
                    params[pm.group(1)] = line_num

                if params:  # Only track invocations with params
                    invokes.append(FrontendInvoke(
                        command=cmd_name,
                        params=params,
                        file=rel_path,
                        line=line_num,
                    ))

    return invokes


def check_mismatches(commands: dict, invokes: list) -> list:
    """Check for parameter name mismatches between frontend and Rust."""
    mismatches = []

    for inv in invokes:
        if inv.command not in commands:
            # Command doesn't exist in Rust — may be a stub or missing command
            continue

        rust_cmd = commands[inv.command]
        issues = []

        if rust_cmd.has_rename_all_camel:
            # Rust expects camelCase — check frontend sends camelCase
            rust_params_camel = [snake_to_camel(p) for p in rust_cmd.params]
            for fp in inv.params:
                if fp not in rust_params_camel:
                    # Check if it's snake_case when it should be camelCase
                    snake_equiv = camel_to_snake(fp)
                    if snake_equiv in rust_cmd.params:
                        issues.append({
                            "frontend_param": fp,
                            "rust_param": snake_equiv,
                            "issue_type": "camel_case_needed",
                            "detail": f"Rust has rename_all=camelCase, expects '{snake_to_camel(snake_equiv)}' but frontend sends '{fp}'",
                        })
        else:
            # Rust expects snake_case — check frontend sends snake_case
            for fp in inv.params:
                if fp not in rust_cmd.params:
                    # Check if it's camelCase when it should be snake_case
                    snake_equiv = camel_to_snake(fp)
                    if snake_equiv in rust_cmd.params:
                        issues.append({
                            "frontend_param": fp,
                            "rust_param": snake_equiv,
                            "issue_type": "snake_case_needed",
                            "detail": f"Rust expects '{snake_equiv}' but frontend sends '{fp}'",
                        })

        if issues:
            mismatches.append(Mismatch(
                invoke=inv,
                rust_cmd=rust_cmd,
                issues=issues,
            ))

    return mismatches


def snake_to_camel(name: str) -> str:
    """Convert snake_case to camelCase."""
    parts = name.split("_")
    return parts[0] + "".join(p.capitalize() for p in parts[1:])


def camel_to_snake(name: str) -> str:
    """Convert camelCase to snake_case."""
    import re
    s1 = re.sub(r"(.)([A-Z][a-z]+)", r"\1_\2", name)
    return re.sub(r"([a-z0-9])([A-Z])", r"\1_\2", s1).lower()


def main():
    project_root = sys.argv[1] if len(sys.argv) > 1 else "."
    project_root = find_project_root(project_root)

    print("=" * 60)
    print("  Tauri Invoke Parameter Checker")
    print("=" * 60)
    print()

    commands = parse_rust_commands(project_root)
    invokes = parse_frontend_invokes(project_root)

    print(f"  Rust commands found: {len(commands)}")
    print(f"  Frontend invoke() calls with params: {len(invokes)}")
    print()

    mismatches = check_mismatches(commands, invokes)

    if not mismatches:
        print("  ✅ No parameter name mismatches found")
        print()
        print("=" * 60)
        print("  🎉 ALL CLEAR")
        print("=" * 60)
        return 0

    print(f"  🔴 Found {len(mismatches)} mismatch(es):\n")

    for mm in mismatches:
        print(f"  📁 {mm.invoke.file}:{mm.invoke.line}")
        print(f"     invoke('{mm.invoke.command}', {{...}})")
        for issue in mm.issues:
            print(f"     ⚠️  {issue['detail']}")
        print(f"     📍 Rust signature: {mm.rust_cmd.file}:{mm.rust_cmd.line}")
        print(f"        fn {mm.rust_cmd.name}({', '.join(mm.rust_cmd.params)})")
        print()

    print("=" * 60)
    print(f"  🔴 {len(mismatches)} MISMATCH(ES) FOUND")
    print("=" * 60)
    return 1


if __name__ == "__main__":
    sys.exit(main())
