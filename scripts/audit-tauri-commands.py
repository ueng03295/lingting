#!/usr/bin/env python3
"""
Tauri Command Audit Script for LingListen (翎听)
Scans #[tauri::command] functions vs frontend invoke() calls.
Usage: python3 scripts/audit-tauri-commands.py
"""

import re
import sys
import json
from pathlib import Path

PROJECT_ROOT = Path(__file__).resolve().parent.parent
RUST_API = PROJECT_ROOT / "frontend/src-tauri/src/api/mod.rs"
FRONTEND_SRC = PROJECT_ROOT / "frontend/src"


def find_tauri_commands(filepath: Path) -> list[dict]:
    """Extract all #[tauri::command] function signatures."""
    commands = []
    text = filepath.read_text()
    lines = text.split('\n')

    i = 0
    while i < len(lines):
        line = lines[i].strip()
        if line == '#[tauri::command]':
            # Collect full signature until opening brace
            sig_parts = []
            j = i + 1
            while j < len(lines):
                sig_parts.append(lines[j])
                if '{' in lines[j]:
                    break
                j += 1
            sig = ' '.join(sig_parts).split('{')[0].strip()

            m = re.match(r'(?:pub\s+)?(?:async\s+)?fn\s+(\w+)', sig)
            if not m:
                i = j
                continue
            fn_name = m.group(1)

            # Extract params with paren balancing
            params = []
            depth = 0
            start = sig.index('(')
            for k in range(start, len(sig)):
                if sig[k] == '(':
                    depth += 1
                elif sig[k] == ')':
                    depth -= 1
                    if depth == 0:
                        param_str = sig[start + 1:k]
                        for p in param_str.split(','):
                            p = p.strip()
                            if not p:
                                continue
                            name = p.split(':')[0].strip().lstrip('_')
                            params.append(name)
                        break

            ret_match = re.search(r'->\s*(.+)', sig)
            ret_type = ret_match.group(1).strip() if ret_match else '()'

            # Read body
            body_lines = lines[j: j + 8]
            body_text = ' '.join(body_lines)
            # Also read more for context (up to 20 lines)
            body_full = ' '.join(lines[j: j + 20])

            is_stub = check_stub(body_text, body_full, fn_name)

            commands.append({
                'name': fn_name,
                'params': params,
                'return_type': ret_type,
                'line': i + 1,
                'is_stub': is_stub,
            })
            i = j
        else:
            i += 1

    return commands


def check_stub(body: str, body_full: str, fn_name: str) -> str | bool:
    """Check if function body looks like a stub (no real side effects)."""
    b = body.replace(' ', '').replace('\n', '')
    full = body_full.replace(' ', '').replace('\n', '')

    # Has real database/repository/command calls → not a stub
    has_real_work = any(kw in full for kw in ['sqlx', 'query', '.pool()', 'Repository', 'spawn(', 'Command::new'])
    if has_real_work:
        return False

    if re.search(r'Ok\(\(\)\)\s*;?\s*}', b):
        if fn_name in ('parakeet_init', 'parakeet_load_model', 'whisper_init', 'whisper_load_model'):
            return False  # These are intentionally bypassed for qwen3-asr
        return 'EMPTY_STUB: returns Ok(()) without side effects'

    if re.search(r'Ok\(serde_json::json!\s*\(\s*\{\s*\}\s*\)\s*\)', b):
        return 'EMPTY_JSON_STUB: returns empty JSON object'

    if re.search(r'Ok\(vec!\[\]\)', b):
        return 'EMPTY_VEC_STUB: returns empty vector'

    if '"connected":false' in b:
        return 'HARDCODED_FAILURE: always returns connected=false'

    return False


def find_frontend_invokes(base_dir: Path) -> list[dict]:
    """Find all invoke() calls in frontend TypeScript."""
    invokes = []
    for ts_file in sorted(base_dir.rglob('*.ts')):
        if 'node_modules' in str(ts_file):
            continue
        text = ts_file.read_text(encoding='utf-8', errors='ignore')
        for m in re.finditer(r"invoke(?:Tauri)?\s*<[^>]*>?\s*\(\s*'([^']+)'\s*,\s*\{", text):
            cmd = m.group(1)
            rel = ts_file.relative_to(base_dir)
            invokes.append({'command': cmd, 'file': str(rel)})
    return invokes


def audit():
    print("=" * 60)
    print("  LingListen Tauri Command Audit")
    print("=" * 60)

    # 1. Rust commands
    rust_api = find_tauri_commands(RUST_API)
    # Also check other command files
    other_cmds = []
    for rs_file in sorted((PROJECT_ROOT / "frontend/src-tauri/src").rglob("commands.rs")):
        other_cmds.extend(find_tauri_commands(rs_file))
    all_rust = {c['name']: c for c in rust_api + other_cmds}

    print(f"\nRust commands: {len(all_rust)}")

    # 2. Stub detection (deduplicated)
    stubs = {}
    for cmd in all_rust.values():
        if cmd['is_stub']:
            key = cmd['name']
            if key not in stubs:
                stubs[key] = cmd

    print(f"\n--- Stubs ---")
    if stubs:
        for name, cmd in sorted(stubs.items()):
            print(f"  ⚠️  {name:45s} {cmd['is_stub']}")
    else:
        print("  ✅ None")

    # 3. Frontend invokes
    frontend = find_frontend_invokes(FRONTEND_SRC)
    print(f"\nFrontend invoke() calls: {len(frontend)}")

    # 4. Cross-reference (deduplicated)
    mismatches = {}
    missing = {}
    for inv in frontend:
        cmd = inv['command']
        if cmd not in all_rust:
            missing[cmd] = inv
            continue
        rust = all_rust[cmd]
        # Only check params that are in invoke's line
        rust_params = [p for p in rust['params'] if p not in ('app', 'state', 'AppState>', 'State')]
        # We can't easily extract the invoke params from a simple match,
        # so we check camelCase → snake_case conversion for known patterns
        # Read the actual invoke line to get params
        text = Path(PROJECT_ROOT / "frontend/src" / inv['file']).read_text(encoding='utf-8', errors='ignore')
        for line in text.split('\n'):
            if f"invoke('{cmd}'" in line or f"invokeTauri('{cmd}'" in line:
                # Find params within the object
                obj_match = re.search(r'\{([^}]+)\}', line)
                if obj_match:
                    obj = obj_match.group(1)
                    fp = re.findall(r'\b(\w+)\s*:', obj)
                    for p in fp:
                        snake = re.sub(r'([A-Z])', r'_\1', p).lower()
                        if snake not in rust_params:
                            key = f"{cmd}:{p}"
                            if key not in mismatches:
                                mismatches[key] = {
                                    'command': cmd, 'frontend_param': p, 'snake': snake,
                                    'rust_params': rust_params, 'file': inv['file']
                                }
                break

    print(f"\n--- Parameter Mismatches ---")
    if mismatches:
        for key, m in sorted(mismatches.items()):
            print(f"  🔴 {m['command']}")
            print(f"     Frontend: '{m['frontend_param']}' → snake: '{m['snake']}'")
            print(f"     Rust expects: {m['rust_params']}")
            print(f"     File: {m['file']}")
    else:
        print("  ✅ None")

    print(f"\n--- Missing Commands ---")
    if missing:
        for cmd, inv in sorted(missing.items()):
            print(f"  🔴 '{cmd}' called but no Rust command found")
            print(f"     File: {inv['file']}")
    else:
        print("  ✅ None")

    # Summary
    print()
    print("=" * 60)
    critical = len(mismatches) + len(missing)
    stub_count = len(stubs)
    if critical == 0 and stub_count == 0:
        print("  🎉 ALL CLEAN")
        print("=" * 60)
        return 0
    else:
        print(f"  {stub_count} stubs, {critical} param/missing issues")
        print("=" * 60)
        return 1


if __name__ == '__main__':
    sys.exit(audit())
