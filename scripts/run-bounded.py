#!/usr/bin/env python3
"""Run a command with a wall-time bound and kill its whole process group."""

from __future__ import annotations

import argparse
import os
import signal
import subprocess
import sys
import time
from collections.abc import Sequence


TIMEOUT_EXIT_CODE = 124
SIGNAL_EXIT_OFFSET = 128


def parse_args(argv: Sequence[str]) -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description=(
            "Run a command with a wall-time timeout. On timeout or wrapper "
            "termination, the whole child process group is terminated."
        )
    )
    parser.add_argument(
        "--timeout",
        type=float,
        required=True,
        help="wall timeout in seconds",
    )
    parser.add_argument(
        "--grace",
        type=float,
        default=2.0,
        help="seconds to wait after SIGTERM before SIGKILL",
    )
    parser.add_argument(
        "command",
        nargs=argparse.REMAINDER,
        help="command to run, optionally preceded by --",
    )
    args = parser.parse_args(argv)

    if args.command and args.command[0] == "--":
        args.command = args.command[1:]
    if not args.command:
        parser.error("expected a command after --")
    if args.timeout <= 0:
        parser.error("--timeout must be positive")
    if args.grace < 0:
        parser.error("--grace must be non-negative")

    return args


def terminate_process_group(process: subprocess.Popen[object], grace: float) -> None:
    if process.poll() is not None:
        return

    try:
        os.killpg(process.pid, signal.SIGTERM)
    except ProcessLookupError:
        return

    deadline = time.monotonic() + grace
    while process.poll() is None and time.monotonic() < deadline:
        time.sleep(0.05)

    if process.poll() is not None:
        return

    try:
        os.killpg(process.pid, signal.SIGKILL)
    except ProcessLookupError:
        return
    process.wait()


def exit_code_for_signal(signum: int) -> int:
    return SIGNAL_EXIT_OFFSET + signum


def main(argv: Sequence[str]) -> int:
    args = parse_args(argv)

    process = subprocess.Popen(args.command, start_new_session=True)

    def signal_handler(signum: int, _frame: object) -> None:
        terminate_process_group(process, args.grace)
        raise SystemExit(exit_code_for_signal(signum))

    previous_handlers = {
        signal.SIGTERM: signal.getsignal(signal.SIGTERM),
        signal.SIGINT: signal.getsignal(signal.SIGINT),
    }
    signal.signal(signal.SIGTERM, signal_handler)
    signal.signal(signal.SIGINT, signal_handler)

    try:
        try:
            return process.wait(timeout=args.timeout)
        except subprocess.TimeoutExpired:
            print(
                f"run-bounded: timed out after {args.timeout:g}s: "
                + " ".join(args.command),
                file=sys.stderr,
            )
            terminate_process_group(process, args.grace)
            return TIMEOUT_EXIT_CODE
    finally:
        for signum, handler in previous_handlers.items():
            signal.signal(signum, handler)


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
