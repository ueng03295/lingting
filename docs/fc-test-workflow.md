# LingListen 云端测试工作流

## 架构概览

```
本地开发 (Ling)          阿里云 FC 沙箱 (Jing 测试)
───────────────────      ──────────────────────────
修改代码 ──┐
           │              ┌── 部署 ──┐
           ▼              │          ▼
    打包 zip ───────────▶│  函数 test-runner
           │              │          │
           └── CI (GitHub)│          ▼
                          │     运行测试
                          │          │
                          │          ▼
                          │     返回结果 (PASS/FAIL)
                          └──────────┘
```

## 当前资源

| 资源 | 值 |
|------|-----|
| 阿里云 CLI Profile | `default` (AK, RAM: test-platform) |
| FC 服务 | `linglisten-test` (`cn-hangzhou`) |
| FC 函数 | `test-runner` (python3.10, 256MB, 60s timeout) |
| 资源包 | FC 试用包 `fc_freecu_dp_cn-adx4sjvda0001` (15 万 CU/月) |

## 1. 快速验证（一行命令）

```bash
# Ping 心跳
aliyun fc-open POST /2021-04-06/services/linglisten-test/functions/test-runner/invocations \
  --body '{"cmd":"ping"}'

# 运行烟雾测试
aliyun fc-open POST /2021-04-06/services/linglisten-test/functions/test-runner/invocations \
  --body '{"cmd":"run_tests"}'
```

预期输出：`"exit_code": 0, "status": "pass"`

## 2. Ling 开发 → 部署流程

### 2.1 修改代码后打包部署

```bash
cd /Users/q/.openclaw/workspace/ling/projects/linglisten

# 1. 打包测试代码（含入口函数 + smoke-test.py）
scripts/deploy-fc.sh
```

`scripts/deploy-fc.sh` 脚本内容：

```bash
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
```

### 2.2 部署后自动验证

脚本会自动运行一次 `run_tests`，确保部署成功后测试通过。

## 3. Jing 测试流程

### 3.1 日常测试

```bash
# 1. 运行烟雾测试
aliyun fc-open POST /2021-04-06/services/linglisten-test/functions/test-runner/invocations \
  --body '{"cmd":"run_tests"}' | python3 -c "
import json, sys
r = json.load(sys.stdin)
print(f'  Status: {r[\"status\"]}')
print(f'  Exit Code: {r[\"exit_code\"]}')
print(f'  Message: {r[\"message\"]}')
print()
print(r.get('stdout', ''))
if r.get('stderr'):
    print('STDERR:', r['stderr'])
if r['exit_code'] != 0:
    sys.exit(1)
"

# 2. 检查服务列表（确认环境正常）
aliyun fc-open GET /2021-04-06/services/linglisten-test
```

### 3.2 验收检查清单

Jing 在收到 Ling 的部署通知后：

1. ✅ 运行 `run_tests`，确认 7/7 PASS
2. ✅ 检查 `exit_code == 0`
3. ✅ 检查 `stderr` 为空
4. ✅ 如新增测试用例，确认数量增加

### 3.3 测试失败时

```bash
# 查看详细输出
aliyun fc-open POST /2021-04-06/services/linglisten-test/functions/test-runner/invocations \
  --body '{"cmd":"run_tests"}' | python3 -m json.tool
```

## 4. 函数入口代码

```python
# deploy-fc-entrypoint.py (GitHub CI 使用的独立版本)
# -*- coding: utf-8 -*-
"""FC 入口 —— 运行 LingListen smoke-test.py"""
import json, os, sys, subprocess

def handler(event, context):
    if isinstance(event, bytes):
        event = event.decode('utf-8')
    try:
        params = json.loads(event) if event else {}
    except:
        params = {}

    cmd = params.get("cmd", "run_tests")

    if cmd == "ping":
        return json.dumps({"status":"ok","message":"FC sandbox alive","cmd":"ping"}, indent=2)

    if cmd == "run_tests":
        result = {"function":"linglisten-smoke-runner","cmd":"run_tests","region":os.environ.get("FC_REGION","unknown")}
        try:
            test_file = os.path.join(os.path.dirname(__file__), 'smoke_test.py')
            proc = subprocess.run(
                [sys.executable, test_file],
                capture_output=True, text=True, timeout=60,
                cwd=os.path.dirname(__file__)
            )
            result["stdout"] = proc.stdout
            result["stderr"] = proc.stderr
            result["exit_code"] = proc.returncode
            result["status"] = "pass" if proc.returncode == 0 else "fail"
            result["message"] = "All tests passed" if proc.returncode == 0 else "Some tests failed"
        except subprocess.TimeoutExpired:
            result["status"] = "error"
            result["message"] = "Tests timed out (60s)"
        except Exception as e:
            result["status"] = "error"
            result["message"] = str(e)
        return json.dumps(result, indent=2)

    return json.dumps({"status":"error","message":f"Unknown command: {cmd}"}, indent=2)
```

## 6. 故障排除

| 问题 | 解决方案 |
|------|----------|
| `AccessDenied` | 检查 test-platform 是否有 `AliyunFCFullAccess` 权限 |
| `can not find api by path` | FC 使用 `fc-open` 不是 `fc`，检查命令前缀 |
| `Tests timed out` | 增加 timeout 参数（当前 60s） |
| `exit_code != 0` | 检查 stderr 输出，查看测试失败原因 |
| CLI 版本问题 | 确保 aliyun >= 3.3.0 |

## 7. 清理资源

测试完成后如需清理：

```bash
# 删除函数
aliyun fc-open DELETE /2021-04-06/services/linglisten-test/functions/test-runner

# 删除服务
aliyun fc-open DELETE /2021-04-06/services/linglisten-test
```
