"""Deterministic, keyless loopback provider for Markion AI acceptance.

Run: python scripts/ai-mock-provider.py --port 18114
Configure a Custom profile: http://127.0.0.1:18114/v1, chat, mock-agent.
mock-text rejects tools. 'slow' produces a long cancellable answer.
Only synthetic acceptance fixtures should be attached to this server.
"""
import argparse
import json
import time
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer


class Handler(BaseHTTPRequestHandler):
    def log_message(self, *_):
        pass  # Do not log requests, document content, or credentials.

    def body(self, value, status=200):
        encoded = json.dumps(value, ensure_ascii=False).encode()
        self.send_response(status)
        self.send_header("Content-Type", "application/json")
        self.send_header("Content-Length", str(len(encoded)))
        self.end_headers()
        self.wfile.write(encoded)

    def do_GET(self):
        if self.path == "/v1/models":
            self.body({"data": [{"id": "mock-agent"}, {"id": "mock-text"}]})
        else:
            self.body({"error": "not found"}, 404)

    def do_POST(self):
        if self.path != "/v1/chat/completions":
            self.body({"error": "not found"}, 404)
            return
        size = int(self.headers.get("Content-Length", "0"))
        if size > 1024 * 1024:
            self.body({"error": "request limit"}, 413)
            return
        request = json.loads(self.rfile.read(size))
        messages = request.get("messages", [])
        user = next((m.get("content", "") for m in reversed(messages) if m.get("role") == "user"), "")
        names = [t["function"]["name"] for t in request.get("tools", [])]
        if names and request.get("model") == "mock-text":
            self.body({"error": "tools unsupported"}, 400)
            return
        calls = []
        text = "OK" if user == "Reply with OK." else "## Mock result\n\n这是本地模拟服务的回答。\n\n- Context is explicit.\n- Review changes before applying."
        if names == ["connection_probe"]:
            calls = [("connection_probe", {"ok": True})]
        elif names and ("organize" in user.lower() or "整理" in user):
            last_user = max((i for i, m in enumerate(messages) if m.get("role") == "user"), default=-1)
            results = [m for m in messages[last_user + 1:] if m.get("role") == "tool"]
            if not results:
                calls = [("list_files", {"offset": 0, "page": 20})]
            elif len(results) == 1:
                calls = [("propose_create_folder", {"path": "organized"})]
            elif len(results) == 2:
                calls = [("propose_move_or_rename", {"path": "a.md", "destination": "organized/a.md"})]
            else:
                text = "The folder and move are proposed. Review the paths and select Apply."
        elif names and ("synthesize" in user.lower() or "综合" in user):
            last_user = max((i for i, m in enumerate(messages) if m.get("role") == "user"), default=-1)
            results = [m for m in messages[last_user + 1:] if m.get("role") == "tool"]
            if not results:
                calls = [("read_text", {"path": "a.md", "start": 0, "end": 4}), ("read_text", {"path": "b.md", "start": 0, "end": 4})]
            else:
                text = "## Synthesis\n\nBoth explicitly granted notes were read."
        if "slow" in user.lower():
            text = "Streaming response for Stop acceptance. " * 50
        tool_calls = [{"index": i, "id": f"mock-call-{i}", "type": "function", "function": {"name": name, "arguments": json.dumps(args)}} for i, (name, args) in enumerate(calls)]
        if not request.get("stream", False):
            self.body({"choices": [{"message": {"role": "assistant", "content": None if calls else text, "tool_calls": tool_calls}, "finish_reason": "tool_calls" if calls else "stop"}]})
            return
        self.send_response(200)
        self.send_header("Content-Type", "text/event-stream")
        self.send_header("Cache-Control", "no-cache")
        self.end_headers()
        try:
            if calls:
                self.frame({"choices": [{"delta": {"tool_calls": tool_calls}, "finish_reason": None}]})
            else:
                for character in text:
                    self.frame({"choices": [{"delta": {"content": character}, "finish_reason": None}]})
                    if "slow" in user.lower():
                        time.sleep(0.04)
            self.frame({"choices": [{"delta": {}, "finish_reason": "tool_calls" if calls else "stop"}]})
            self.wfile.write(b"data: [DONE]\n\n")
            self.wfile.flush()
        except (BrokenPipeError, ConnectionResetError):
            pass

    def frame(self, value):
        encoded = ("data: " + json.dumps(value, ensure_ascii=False) + "\n\n").encode()
        # Deliberately fragment inside a multi-byte character when present.
        split = next((i + 1 for i, byte in enumerate(encoded) if byte >= 0xC0), len(encoded) - 1)
        self.wfile.write(encoded[:split])
        self.wfile.flush()
        self.wfile.write(encoded[split:])
        self.wfile.flush()


if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("--port", type=int, default=18114)
    arguments = parser.parse_args()
    server = ThreadingHTTPServer(("127.0.0.1", arguments.port), Handler)
    print(f"Synthetic Markion provider: http://127.0.0.1:{arguments.port}/v1", flush=True)
    server.serve_forever()
