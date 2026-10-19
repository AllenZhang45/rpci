#!/usr/bin/env python3
"""Serve a textarea page and record its value to /tmp/ime-firefox.txt.

Used for real-application matrix testing of the input method.
"""

from http.server import BaseHTTPRequestHandler, HTTPServer

LOG_PATH = "/tmp/ime-firefox.txt"

PAGE = """<!doctype html>
<html>
<meta charset="utf-8">
<title>IME test</title>
<style>
  html, body { margin: 0; height: 100%; background: #222; }
  textarea {
    box-sizing: border-box;
    width: 100%; height: 100%;
    border: 0; padding: 24px;
    background: #222; color: #eee;
    font: 24px "Noto Sans CJK SC", sans-serif;
    outline: none; resize: none;
  }
</style>
<textarea id="t" autofocus placeholder="type here"></textarea>
<script>
  const t = document.getElementById("t");
  t.focus();
  t.addEventListener("input", async () => {
    await fetch("/log", { method: "POST", body: t.value });
  });
</script>
</html>
"""


class Handler(BaseHTTPRequestHandler):
    def do_GET(self):
        body = PAGE.encode("utf-8")
        self.send_response(200)
        self.send_header("Content-Type", "text/html; charset=utf-8")
        self.send_header("Content-Length", str(len(body)))
        self.end_headers()
        self.wfile.write(body)

    def do_POST(self):
        length = int(self.headers.get("Content-Length", "0"))
        body = self.rfile.read(length)
        with open(LOG_PATH, "wb") as f:
            f.write(body)
        self.send_response(204)
        self.end_headers()

    def log_message(self, *_args):
        pass


if __name__ == "__main__":
    HTTPServer(("127.0.0.1", 8765), Handler).serve_forever()
