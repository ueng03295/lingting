#!/bin/bash
set -e

PROJECT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
TMPDIR=$(mktemp -d)

echo "📦 Building FC test runner..."

# 创建入口函数
cat > "$TMPDIR/index.py" << 'PYEOF'
# -*- coding: utf-8 -*-
"""FC 入口 —— 运行 LingListen smoke-test.py"""
import json, os, sys, subprocess, tempfile, base64

def handler(event, context):
    if isinstance(event, bytes):
        event = event.decode('utf-8')
    try:
        params = json.loads(event) if event else {}
    except:
        params = {}

    cmd = params.get("cmd", "run_tests")

    if cmd == "ping":
        return json.dumps({"function":"linglisten-smoke-runner","status":"ok","message":"FC sandbox alive","cmd":"ping"}, indent=2)

    if cmd == "run_tests":
        result = {"function":"linglisten-smoke-runner","cmd":"run_tests","region":os.environ.get("FC_REGION","unknown")}
        try:
            test_file = os.path.join(os.path.dirname(__file__), 'smoke_test.py')
            proc = subprocess.run([sys.executable, test_file], capture_output=True, text=True, timeout=60, cwd=os.path.dirname(__file__))
            result["stdout"] = proc.stdout
            result["stderr"] = proc.stderr
            result["exit_code"] = proc.returncode
            result["status"] = "pass" if proc.returncode == 0 else "fail"
            result["message"] = "All tests passed" if proc.returncode == 0 else "Some tests failed"
        except subprocess.TimeoutExpired:
            result["status"] = "error"; result["message"] = "Tests timed out (60s)"
        except Exception as e:
            result["status"] = "error"; result["message"] = str(e)
        return json.dumps(result, indent=2)

    return json.dumps({"status":"error","message":f"Unknown command: {cmd}"}, indent=2)
PYEOF

# 复制测试脚本
cp "$PROJECT_DIR/scripts/smoke-test.py" "$TMPDIR/smoke_test.py"

cd "$TMPDIR" && zip -q -r "$TMPDIR/deploy.zip" .

echo "🚀 Deploying to FC..."
ZIP_B64=$(base64 -i "$TMPDIR/deploy.zip" | tr -d '\n')

# 删除旧函数 + 创建新函数
aliyun fc-open DELETE /2021-04-06/services/linglisten-test/functions/test-runner 2>/dev/null || true

aliyun fc-open POST /2021-04-06/services/linglisten-test/functions \
  --body "{\"functionName\":\"test-runner\",\"runtime\":\"python3.10\",\"handler\":\"index.handler\",\"code\":{\"zipFile\":\"${ZIP_B64}\"},\"description\":\"LingListen烟雾测试运行器\",\"timeout\":60,\"memorySize\":256}"

# 立即运行测试
echo ""
echo "🧪 Running tests..."
aliyun fc-open POST /2021-04-06/services/linglisten-test/functions/test-runner/invocations \
  --body '{"cmd":"run_tests"}'

rm -rf "$TMPDIR"
