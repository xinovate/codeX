#!/usr/bin/env python3
"""
Codex 国内平台协议转换验证脚本

验证 Responses API ↔ Chat API 的请求/响应转换逻辑。
用真实 API Key 测试，确认转换正确后再移植到 Rust。

用法:
    python3 validate_conversion.py
"""

import json
import os
import sys
import requests

# ============================================================
# 配置
# ============================================================

VOLCENGINE_API_KEY = os.environ.get(
    "VOLCENGINE_API_KEY",
    "8d089fdf-4cac-4fc4-937a-5b6be3aa7bab"
)
VOLCENGINE_BASE_URL = "https://ark.cn-beijing.volces.com/api/coding/v3"
TEST_MODEL = "doubao-seed-2.0-pro"

# ============================================================
# 转换逻辑
# ============================================================

def convert_request(responses_req: dict) -> dict:
    """将 Responses API 请求转换为 Chat API 请求"""
    chat_req = {}

    # 1. input -> messages
    if "input" in responses_req:
        chat_req["messages"] = responses_req["input"]

    # 2. max_output_tokens -> max_tokens
    if "max_output_tokens" in responses_req:
        chat_req["max_tokens"] = responses_req["max_output_tokens"]

    # 3. text.format -> response_format
    if "text" in responses_req and "format" in responses_req["text"]:
        chat_req["response_format"] = responses_req["text"]["format"]

    # 4. 透传其他字段
    skip_keys = {"input", "max_output_tokens", "text"}
    for key, value in responses_req.items():
        if key not in skip_keys:
            chat_req[key] = value

    return chat_req


def map_finish_reason(reason: str) -> str:
    """将 Chat API finish_reason 映射为 Responses API status"""
    mapping = {
        "stop": "completed",
        "length": "incomplete",
        "tool_calls": "requires_action",
        "content_filter": "incomplete",
    }
    return mapping.get(reason, "in_progress")


def convert_response(chat_resp: dict) -> dict:
    """将 Chat API 响应转换为 Responses API 响应"""
    responses_resp = {}

    # 1. id
    if "id" in chat_resp:
        responses_resp["id"] = chat_resp["id"]

    # 2. object
    responses_resp["object"] = "response"

    # 3. created_at
    if "created" in chat_resp:
        responses_resp["created_at"] = chat_resp["created"]

    # 4. model
    if "model" in chat_resp:
        responses_resp["model"] = chat_resp["model"]

    # 5. choices -> output
    if "choices" in chat_resp:
        output_items = []
        for choice in chat_resp["choices"]:
            output_item = {"type": "message"}

            if "message" in choice:
                msg = choice["message"]

                # role
                if "role" in msg:
                    output_item["role"] = msg["role"]

                # content -> content[{type, text}]
                if "content" in msg and msg["content"]:
                    output_item["content"] = [
                        {"type": "output_text", "text": msg["content"]}
                    ]

                # tool_calls
                if "tool_calls" in msg:
                    output_item["content"] = output_item.get("content", [])
                    for tc in msg["tool_calls"]:
                        output_item["content"].append({
                            "type": "function_call",
                            "id": tc.get("id", ""),
                            "call_id": tc.get("id", ""),
                            "name": tc["function"]["name"],
                            "arguments": tc["function"]["arguments"],
                        })

            # finish_reason -> status
            if "finish_reason" in choice and choice["finish_reason"]:
                output_item["status"] = map_finish_reason(choice["finish_reason"])

            output_items.append(output_item)

        responses_resp["output"] = output_items

    # 6. usage
    if "usage" in chat_resp:
        usage = chat_resp["usage"]
        converted_usage = {}
        if "prompt_tokens" in usage:
            converted_usage["input_tokens"] = usage["prompt_tokens"]
        if "completion_tokens" in usage:
            converted_usage["output_tokens"] = usage["completion_tokens"]
        if "total_tokens" in usage:
            converted_usage["total_tokens"] = usage["total_tokens"]
        responses_resp["usage"] = converted_usage

    return responses_resp


def convert_stream_chunk(chat_chunk: dict) -> dict:
    """将 Chat API SSE 流块转换为 Responses API 流块"""
    responses_chunk = {}

    if "choices" not in chat_chunk:
        return responses_chunk

    choices = chat_chunk["choices"]
    if not choices:
        return responses_chunk

    first_choice = choices[0]
    output_item = {"type": "message"}

    # delta.content -> content[{type, text}]
    if "delta" in first_choice:
        delta = first_choice["delta"]

        # 处理 content（可能为空字符串，如火山引擎的 reasoning 阶段）
        content_value = delta.get("content")
        if content_value:  # 非空字符串
            output_item["content"] = [
                {"type": "output_text", "text": content_value}
            ]

        # 处理 reasoning_content（火山引擎特有，转换为 reasoning 类型）
        reasoning_value = delta.get("reasoning_content")
        if reasoning_value:
            if "content" not in output_item:
                output_item["content"] = []
            output_item["content"].append({
                "type": "reasoning",
                "text": reasoning_value
            })

        if "role" in delta:
            output_item["role"] = delta["role"]

        # 工具调用
        tool_calls = delta.get("tool_calls")
        if tool_calls and isinstance(tool_calls, list):
            if "content" not in output_item:
                output_item["content"] = []
            for tc in tool_calls:
                tc_item = {
                    "type": "function_call",
                    "index": tc.get("index", 0),
                }
                if "id" in tc:
                    tc_item["id"] = tc["id"]
                    tc_item["call_id"] = tc["id"]
                if "function" in tc:
                    if "name" in tc["function"]:
                        tc_item["name"] = tc["function"]["name"]
                    if "arguments" in tc["function"]:
                        tc_item["arguments"] = tc["function"]["arguments"]
                output_item["content"].append(tc_item)

    # finish_reason -> status
    if "finish_reason" in first_choice and first_choice["finish_reason"]:
        output_item["status"] = map_finish_reason(first_choice["finish_reason"])

    responses_chunk["output"] = [output_item]

    # usage (可能在最后一个 chunk，火山引擎可能返回 null)
    if "usage" in chat_chunk and chat_chunk["usage"] is not None and isinstance(chat_chunk["usage"], dict):
        usage = chat_chunk["usage"]
        converted_usage = {}
        if "prompt_tokens" in usage:
            converted_usage["input_tokens"] = usage["prompt_tokens"]
        if "completion_tokens" in usage:
            converted_usage["output_tokens"] = usage["completion_tokens"]
        responses_chunk["usage"] = converted_usage

    return responses_chunk


# ============================================================
# 测试用例
# ============================================================

def test_request_conversion():
    """测试请求转换逻辑"""
    print("=" * 60)
    print("测试 1: 请求转换 (Responses → Chat)")
    print("=" * 60)

    responses_req = {
        "model": "doubao-seed-2.0-pro",
        "input": [
            {"role": "system", "content": "You are a helpful assistant."},
            {"role": "user", "content": "Hello"}
        ],
        "max_output_tokens": 100,
        "temperature": 0.7,
    }

    chat_req = convert_request(responses_req)

    print(f"\n输入 (Responses API):\n{json.dumps(responses_req, indent=2, ensure_ascii=False)}")
    print(f"\n输出 (Chat API):\n{json.dumps(chat_req, indent=2, ensure_ascii=False)}")

    # 验证
    assert "messages" in chat_req, "缺少 messages 字段"
    assert chat_req["messages"] == responses_req["input"], "messages 转换错误"
    assert "max_tokens" in chat_req, "缺少 max_tokens 字段"
    assert chat_req["max_tokens"] == 100, "max_tokens 转换错误"
    assert "model" in chat_req, "缺少 model 字段"
    assert "input" not in chat_req, "input 字段应被移除"
    assert "max_output_tokens" not in chat_req, "max_output_tokens 字段应被移除"

    print("\n✅ 请求转换测试通过")
    return True


def test_response_conversion():
    """测试响应转换逻辑"""
    print("\n" + "=" * 60)
    print("测试 2: 响应转换 (Chat → Responses)")
    print("=" * 60)

    chat_resp = {
        "id": "chatcmpl-123",
        "object": "chat.completion",
        "created": 1234567890,
        "model": "doubao-seed-2.0-pro",
        "choices": [
            {
                "index": 0,
                "message": {
                    "role": "assistant",
                    "content": "Hello! How can I help you today?"
                },
                "finish_reason": "stop"
            }
        ],
        "usage": {
            "prompt_tokens": 10,
            "completion_tokens": 8,
            "total_tokens": 18
        }
    }

    responses_resp = convert_response(chat_resp)

    print(f"\n输入 (Chat API):\n{json.dumps(chat_resp, indent=2, ensure_ascii=False)}")
    print(f"\n输出 (Responses API):\n{json.dumps(responses_resp, indent=2, ensure_ascii=False)}")

    # 验证
    assert "output" in responses_resp, "缺少 output 字段"
    assert responses_resp["output"][0]["role"] == "assistant", "role 转换错误"
    assert responses_resp["output"][0]["content"][0]["text"] == "Hello! How can I help you today?", "content 转换错误"
    assert responses_resp["output"][0]["status"] == "completed", "status 转换错误"
    assert responses_resp["usage"]["input_tokens"] == 10, "usage 转换错误"
    assert responses_resp["usage"]["output_tokens"] == 8, "usage 转换错误"

    print("\n✅ 响应转换测试通过")
    return True


def test_stream_chunk_conversion():
    """测试流式响应转换逻辑"""
    print("\n" + "=" * 60)
    print("测试 3: 流式响应转换 (Chat SSE → Responses SSE)")
    print("=" * 60)

    # 模拟 Chat API 的 SSE 流
    chat_chunks = [
        {"choices": [{"delta": {"role": "assistant"}, "index": 0}]},
        {"choices": [{"delta": {"content": "Hello"}, "index": 0}]},
        {"choices": [{"delta": {"content": "!"}, "index": 0}]},
        {"choices": [{"delta": {}, "finish_reason": "stop", "index": 0}]},
    ]

    print("\nChat SSE 流块 → Responses SSE 流块:")
    for i, chunk in enumerate(chat_chunks):
        converted = convert_stream_chunk(chunk)
        print(f"\n  块 {i+1}:")
        print(f"    输入: {json.dumps(chunk, ensure_ascii=False)}")
        print(f"    输出: {json.dumps(converted, ensure_ascii=False)}")

    # 验证最后一个块
    last_converted = convert_stream_chunk(chat_chunks[-1])
    assert last_converted["output"][0]["status"] == "completed", "流式 status 转换错误"

    print("\n✅ 流式响应转换测试通过")
    return True


def test_finish_reason_mapping():
    """测试 finish_reason 映射"""
    print("\n" + "=" * 60)
    print("测试 4: finish_reason 映射")
    print("=" * 60)

    test_cases = [
        ("stop", "completed"),
        ("length", "incomplete"),
        ("tool_calls", "requires_action"),
        ("content_filter", "incomplete"),
        (None, "in_progress"),
        ("unknown", "in_progress"),
    ]

    all_passed = True
    for reason, expected in test_cases:
        if reason is None:
            actual = map_finish_reason("")
        else:
            actual = map_finish_reason(reason)
        status = "✅" if actual == expected else "❌"
        print(f"  {status} {reason!r} → {actual!r} (期望: {expected!r})")
        if actual != expected:
            all_passed = False

    assert all_passed, "finish_reason 映射有误"
    print("\n✅ finish_reason 映射测试通过")
    return True


# ============================================================
# 真实 API 测试
# ============================================================

def test_real_api():
    """用真实 API 测试完整流程"""
    print("\n" + "=" * 60)
    print("测试 5: 真实 API 调用 (火山引擎)")
    print("=" * 60)

    # 1. 构造 Responses 格式请求（模拟 Codex 发出的）
    responses_req = {
        "model": TEST_MODEL,
        "input": [
            {"role": "user", "content": "用一句话介绍你自己"}
        ],
        "max_output_tokens": 100,
    }

    print(f"\n1. 原始请求 (Responses 格式):\n{json.dumps(responses_req, indent=2, ensure_ascii=False)}")

    # 2. 转换为 Chat 格式
    chat_req = convert_request(responses_req)
    print(f"\n2. 转换后请求 (Chat 格式):\n{json.dumps(chat_req, indent=2, ensure_ascii=False)}")

    # 3. 发送请求
    url = f"{VOLCENGINE_BASE_URL}/chat/completions"
    headers = {
        "Authorization": f"Bearer {VOLCENGINE_API_KEY}",
        "Content-Type": "application/json",
    }

    print(f"\n3. 发送请求到: {url}")
    try:
        resp = requests.post(url, headers=headers, json=chat_req, timeout=30)
        print(f"   HTTP 状态: {resp.status_code}")

        if resp.status_code != 200:
            print(f"   ❌ 请求失败: {resp.text[:500]}")
            return False

        chat_resp = resp.json()
        print(f"\n4. Chat API 响应:\n{json.dumps(chat_resp, indent=2, ensure_ascii=False)[:1000]}")

    except Exception as e:
        print(f"   ❌ 请求异常: {e}")
        return False

    # 4. 转换回 Responses 格式
    responses_resp = convert_response(chat_resp)
    print(f"\n5. 转换后响应 (Responses 格式):\n{json.dumps(responses_resp, indent=2, ensure_ascii=False)}")

    # 5. 验证
    assert "output" in responses_resp, "缺少 output 字段"
    assert len(responses_resp["output"]) > 0, "output 为空"
    assert "content" in responses_resp["output"][0], "缺少 content 字段"
    assert len(responses_resp["output"][0]["content"]) > 0, "content 为空"

    text = responses_resp["output"][0]["content"][0].get("text", "")
    print(f"\n6. 提取文本: {text}")

    print("\n✅ 真实 API 测试通过")
    return True


def test_real_api_stream():
    """用真实 API 测试流式响应"""
    print("\n" + "=" * 60)
    print("测试 6: 真实 API 流式调用 (火山引擎)")
    print("=" * 60)

    # 1. 构造请求
    responses_req = {
        "model": TEST_MODEL,
        "input": [
            {"role": "user", "content": "数到5"}
        ],
        "max_output_tokens": 100,
    }

    chat_req = convert_request(responses_req)
    chat_req["stream"] = True

    # 2. 发送流式请求
    url = f"{VOLCENGINE_BASE_URL}/chat/completions"
    headers = {
        "Authorization": f"Bearer {VOLCENGINE_API_KEY}",
        "Content-Type": "application/json",
    }

    print(f"\n发送流式请求到: {url}")
    try:
        resp = requests.post(url, headers=headers, json=chat_req, timeout=30, stream=True)
        print(f"HTTP 状态: {resp.status_code}")

        if resp.status_code != 200:
            print(f"❌ 请求失败: {resp.text[:500]}")
            return False

    except Exception as e:
        print(f"❌ 请求异常: {e}")
        return False

    # 3. 处理 SSE 流
    print("\n流式输出 (Chat → Responses 转换):")
    print("-" * 40)

    full_text = ""
    chunk_count = 0

    for line in resp.iter_lines():
        if not line:
            continue

        line = line.decode("utf-8")
        if not line.startswith("data: "):
            continue

        data = line[6:]
        if data.strip() == "[DONE]":
            print("\n  [DONE]")
            break

        try:
            chat_chunk = json.loads(data)
        except json.JSONDecodeError:
            continue

        # 调试: 打印原始 chunk
        # print(f"\n  [DEBUG chunk]: {json.dumps(chat_chunk, ensure_ascii=False)[:200]}")

        # 转换
        try:
            responses_chunk = convert_stream_chunk(chat_chunk)
        except Exception as e:
            print(f"\n  [转换错误]: {e}")
            print(f"  [原始数据]: {json.dumps(chat_chunk, ensure_ascii=False)[:300]}")
            continue
        chunk_count += 1

        # 提取文本
        if "output" in responses_chunk and responses_chunk["output"]:
            output_item = responses_chunk["output"][0]
            content = output_item.get("content")
            # content 可能是 None 或空列表
            if content and isinstance(content, list) and len(content) > 0:
                first_content = content[0]
                if isinstance(first_content, dict) and "text" in first_content:
                    text = first_content["text"]
                    full_text += text
                    print(text, end="", flush=True)

            status = output_item.get("status")
            if status:
                print(f"\n  [status: {status}]")

    print(f"\n\n完整文本: {full_text}")
    print(f"流块数: {chunk_count}")

    if full_text:
        print("\n✅ 流式 API 测试通过")
        return True
    else:
        print("\n❌ 流式 API 测试失败: 未收到文本")
        return False


# ============================================================
# 主函数
# ============================================================

def main():
    print("Codex 国内平台协议转换验证")
    print("=" * 60)

    results = {}

    # 纯逻辑测试（不需要网络）
    try:
        results["请求转换"] = test_request_conversion()
    except AssertionError as e:
        print(f"\n❌ 请求转换测试失败: {e}")
        results["请求转换"] = False

    try:
        results["响应转换"] = test_response_conversion()
    except AssertionError as e:
        print(f"\n❌ 响应转换测试失败: {e}")
        results["响应转换"] = False

    try:
        results["流式转换"] = test_stream_chunk_conversion()
    except AssertionError as e:
        print(f"\n❌ 流式转换测试失败: {e}")
        results["流式转换"] = False

    try:
        results["finish_reason映射"] = test_finish_reason_mapping()
    except AssertionError as e:
        print(f"\n❌ finish_reason 映射测试失败: {e}")
        results["finish_reason映射"] = False

    # 真实 API 测试（需要网络）
    try:
        results["真实API-非流式"] = test_real_api()
    except Exception as e:
        print(f"\n❌ 真实 API 测试失败: {e}")
        results["真实API-非流式"] = False

    try:
        results["真实API-流式"] = test_real_api_stream()
    except Exception as e:
        import traceback
        print(f"\n❌ 流式 API 测试失败: {e}")
        traceback.print_exc()
        results["真实API-流式"] = False

    # 汇总
    print("\n" + "=" * 60)
    print("测试汇总")
    print("=" * 60)

    for name, passed in results.items():
        status = "✅ 通过" if passed else "❌ 失败"
        print(f"  {status}  {name}")

    total = len(results)
    passed = sum(1 for v in results.values() if v)
    print(f"\n  总计: {passed}/{total} 通过")

    if passed == total:
        print("\n🎉 全部通过！转换逻辑正确，可以移植到 Rust。")
        return 0
    else:
        print("\n⚠️ 部分测试失败，需要修复后再移植。")
        return 1


if __name__ == "__main__":
    sys.exit(main())
