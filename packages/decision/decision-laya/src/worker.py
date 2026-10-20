#!/usr/bin/env python3
"""Serve cached Laya typed decisions on loopback and exit after inactivity."""

import argparse
import json
import os
import sys
import threading
import time
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer

os.environ["USE_TF"] = "0"


class IdleWatchdog:
    """Terminate the server after no decision evaluations for the configured interval."""

    def __init__(self, timeout_seconds: float, on_timeout):
        self.timeout_seconds = timeout_seconds
        self.on_timeout = on_timeout
        self.timer = None
        self.lock = threading.Lock()

    def reset(self):
        with self.lock:
            if self.timer is not None:
                self.timer.cancel()
            if self.timeout_seconds > 0:
                self.timer = threading.Timer(self.timeout_seconds, self._fire)
                self.timer.daemon = True
                self.timer.start()

    def cancel(self):
        with self.lock:
            if self.timer is not None:
                self.timer.cancel()
                self.timer = None

    def _fire(self):
        self.on_timeout()


def make_handler(agent_holder, watchdog, server_ref):
    """Create the request handler around mutable worker-owned state."""

    class LayaHandler(BaseHTTPRequestHandler):
        def log_message(self, _format, *_args):
            pass

        def send_json(self, status, payload):
            body = json.dumps(payload).encode("utf-8")
            self.send_response(status)
            self.send_header("Content-Type", "application/json")
            self.send_header("Content-Length", str(len(body)))
            self.end_headers()
            self.wfile.write(body)

        def do_GET(self):
            if self.path not in ("/", "/health"):
                self.send_json(404, {"error": {"message": "not found"}})
                return
            agent = agent_holder.get("agent")
            if agent is None:
                self.send_json(503, {"status": "loading"})
                return
            self.send_json(200, {"status": "ok", "model": agent_holder["model"]})

        def do_POST(self):
            if self.path == "/shutdown":
                self.send_json(200, {"status": "shutting_down"})
                threading.Thread(target=server_ref["shutdown"], daemon=True).start()
                return
            if self.path not in ("/v1/systemone", "/evaluate"):
                self.send_json(404, {"error": {"message": "not found"}})
                return

            watchdog.reset()
            try:
                content_len = int(self.headers.get("Content-Length", 0))
                payload = json.loads(self.rfile.read(content_len).decode("utf-8"))
                questions = {}
                for question_id, definition in payload.get("questions", {}).items():
                    question_type = definition.get("type")
                    if question_type == "boolean":
                        question_type = "noul"
                    question = {
                        "type": question_type,
                        "instructions": definition.get("instructions", ""),
                    }
                    if "criteria" in definition:
                        question["criteria"] = definition["criteria"]
                    questions[question_id] = question
                result = agent_holder["agent"].predict(payload.get("state", ""), questions)
                self.send_json(200, result)
            except Exception as error:
                self.send_json(
                    500,
                    {"error": {"message": str(error), "type": type(error).__name__}},
                )

    return LayaHandler


def main():
    parser = argparse.ArgumentParser(description="Laya System 1 decision worker")
    parser.add_argument("--host", default="127.0.0.1")
    parser.add_argument("--port", type=int, default=58085)
    parser.add_argument("--idle-timeout", type=float, default=300.0)
    parser.add_argument("--model", default="convaiinnovations/laya")
    parser.add_argument("--model-subfolder")
    args = parser.parse_args()

    server_ref = {}
    agent_holder = {"agent": None, "model": args.model}

    def on_timeout():
        server_ref["shutdown"]()

    watchdog = IdleWatchdog(args.idle_timeout, on_timeout)
    handler = make_handler(agent_holder, watchdog, server_ref)
    server = ThreadingHTTPServer((args.host, args.port), handler)
    server_ref["shutdown"] = server.shutdown

    try:
        start = time.perf_counter()
        import laya
        from huggingface_hub import snapshot_download

        model_path = args.model
        if not os.path.exists(model_path):
            patterns = None if args.model_subfolder is None else [f"{args.model_subfolder}/*"]
            model_path = snapshot_download(
                args.model,
                allow_patterns=patterns,
                local_files_only=True,
            )
        agent_holder["agent"] = laya.load(model_path, subfolder=args.model_subfolder)
        watchdog.reset()
        elapsed = time.perf_counter() - start
        print(
            json.dumps(
                {
                    "ready": True,
                    "host": args.host,
                    "port": server.server_address[1],
                    "model": args.model,
                    "subfolder": args.model_subfolder,
                    "load_seconds": round(elapsed, 3),
                }
            ),
            flush=True,
        )
        server.serve_forever()
    finally:
        watchdog.cancel()
        server.server_close()


if __name__ == "__main__":
    main()
