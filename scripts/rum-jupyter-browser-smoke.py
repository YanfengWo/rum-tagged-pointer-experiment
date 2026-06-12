#!/usr/bin/env python3
"""Headless JupyterLab smoke for the development rum kernel."""

from __future__ import annotations

import json
import os
import socket
import subprocess
import sys
import time
import urllib.error
import urllib.request
from pathlib import Path


def env_path(name: str) -> Path:
    value = os.environ.get(name)
    if not value:
        raise SystemExit(f"error: {name} is not set; use scripts/rum-jupyter-home.sh")
    return Path(value)


def free_port() -> int:
    with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as sock:
        sock.bind(("127.0.0.1", 0))
        return int(sock.getsockname()[1])


def wait_for_server(url: str, timeout: float = 60.0) -> None:
    deadline = time.monotonic() + timeout
    while time.monotonic() < deadline:
        try:
            with urllib.request.urlopen(url, timeout=2) as response:
                if response.status == 200:
                    return
        except (urllib.error.URLError, TimeoutError):
            time.sleep(0.25)
    raise RuntimeError(f"Jupyter server did not become ready: {url}")


def write_notebook(path: Path) -> None:
    notebook = {
        "cells": [
            {
                "cell_type": "code",
                "execution_count": None,
                "id": "rum-smoke-cell",
                "metadata": {},
                "outputs": [],
                "source": ["1+1"],
            }
        ],
        "metadata": {
            "kernelspec": {
                "display_name": "rum (dev)",
                "language": "rum",
                "name": "rum-dev",
            },
            "language_info": {"name": "rum"},
        },
        "nbformat": 4,
        "nbformat_minor": 5,
    }
    path.write_text(json.dumps(notebook, indent=2), encoding="utf-8")


def capture_failure(page, artifacts: Path) -> None:
    artifacts.mkdir(parents=True, exist_ok=True)
    screenshot = artifacts / "browser-smoke-failure.png"
    html = artifacts / "browser-smoke-failure.html"
    try:
        page.screenshot(path=str(screenshot), full_page=True)
        print(f"screenshot={screenshot}", file=sys.stderr)
    except Exception as error:
        print(f"screenshot_error={error}", file=sys.stderr)
    try:
        html.write_text(page.content(), encoding="utf-8")
        print(f"html={html}", file=sys.stderr)
    except Exception as error:
        print(f"html_error={error}", file=sys.stderr)


def main() -> int:
    home = env_path("RUM_JUPYTER_HOME")
    python = env_path("RUM_JUPYTER_PYTHON")
    workspace = home / "workspace"
    logs = home / "logs"
    user_data = home / "playwright" / "user-data"
    browser_cache = home / "playwright" / "browser-cache"
    artifacts = home / "playwright" / "artifacts"
    for path in (workspace, logs, user_data, browser_cache, artifacts):
        path.mkdir(parents=True, exist_ok=True)

    notebook_path = workspace / "browser-smoke.ipynb"
    write_notebook(notebook_path)

    token = "rum-browser-smoke"
    port = free_port()
    base_url = f"http://127.0.0.1:{port}"
    status_url = f"{base_url}/api/status?token={token}"
    lab_url = f"{base_url}/lab/tree/{notebook_path.name}?token={token}"

    jupyter = python.parent / "jupyter"
    log_path = logs / "browser-smoke-jupyter.log"
    log_file = log_path.open("w", encoding="utf-8")
    proc = subprocess.Popen(
        [
            str(jupyter),
            "lab",
            "--no-browser",
            "--ServerApp.open_browser=False",
            "--ServerApp.ip=127.0.0.1",
            f"--ServerApp.port={port}",
            "--ServerApp.port_retries=0",
            f"--ServerApp.token={token}",
            "--ServerApp.password=",
            f"--ServerApp.root_dir={workspace}",
            "--LabApp.extension_manager=readonly",
        ],
        cwd=workspace,
        env=os.environ.copy(),
        stdout=log_file,
        stderr=subprocess.STDOUT,
        text=True,
    )

    try:
        wait_for_server(status_url)
        from playwright.sync_api import sync_playwright

        with sync_playwright() as playwright:
            context = playwright.chromium.launch_persistent_context(
                str(user_data),
                headless=True,
                args=[
                    f"--disk-cache-dir={browser_cache}",
                    "--disable-dev-shm-usage",
                    "--no-first-run",
                    "--no-default-browser-check",
                ],
            )
            page = context.new_page()
            try:
                page.goto(lab_url, wait_until="domcontentloaded", timeout=60_000)
                page.wait_for_selector(".jp-Notebook", timeout=60_000)
                page.locator(".jp-Cell .cm-content").first.click()
                page.keyboard.press("Shift+Enter")
                page.wait_for_function(
                    """
                    () => Array.from(document.querySelectorAll('.jp-OutputArea'))
                      .some(node => node.textContent && node.textContent.includes('2'))
                    """,
                    timeout=60_000,
                )
            except Exception:
                capture_failure(page, artifacts)
                raise
            finally:
                context.close()
        print(f"browser_smoke=ok")
        print(f"url={lab_url}")
        print(f"log={log_path}")
        print(f"notebook={notebook_path}")
        return 0
    finally:
        proc.terminate()
        try:
            proc.wait(timeout=10)
        except subprocess.TimeoutExpired:
            proc.kill()
            proc.wait(timeout=10)
        log_file.close()


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except Exception as error:
        print(f"browser_smoke=error: {error}", file=sys.stderr)
        raise
