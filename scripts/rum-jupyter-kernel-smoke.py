#!/usr/bin/env python3
"""Real jupyter_client smoke tests for the development rum kernel."""

from __future__ import annotations

import os
import sys
import time
from dataclasses import dataclass, field
from queue import Empty
from typing import Any, Callable

from jupyter_client import KernelManager


TIMEOUT = 30.0


class SmokeFailure(AssertionError):
    pass


@dataclass
class ExecuteResult:
    code: str
    shell: dict[str, Any] | None = None
    iopub: list[dict[str, Any]] = field(default_factory=list)
    stdin: list[dict[str, Any]] = field(default_factory=list)
    result_text: str | None = None

    @property
    def shell_status(self) -> str | None:
        if self.shell is None:
            return None
        return self.shell.get("content", {}).get("status")

    @property
    def execution_count(self) -> int | None:
        if self.shell is None:
            return None
        return self.shell.get("content", {}).get("execution_count")

    @property
    def iopub_types(self) -> list[str]:
        return [msg["header"]["msg_type"] for msg in self.iopub]

    @property
    def stdin_prompts(self) -> list[str]:
        return [
            msg.get("content", {}).get("prompt", "")
            for msg in self.stdin
            if msg.get("header", {}).get("msg_type") == "input_request"
        ]

    @property
    def stdout_text(self) -> str:
        return "".join(
            msg.get("content", {}).get("text", "")
            for msg in self.iopub
            if msg.get("header", {}).get("msg_type") == "stream"
            and msg.get("content", {}).get("name") == "stdout"
        )

    @property
    def stderr_text(self) -> str:
        return "".join(
            msg.get("content", {}).get("text", "")
            for msg in self.iopub
            if msg.get("header", {}).get("msg_type") == "stream"
            and msg.get("content", {}).get("name") == "stderr"
        )

    @property
    def error_names(self) -> list[str]:
        return [
            msg.get("content", {}).get("ename", "")
            for msg in self.iopub
            if msg.get("header", {}).get("msg_type") == "error"
        ]


def fail(message: str) -> None:
    raise SmokeFailure(message)


def assert_equal(found: Any, expected: Any, label: str) -> None:
    if found != expected:
        fail(f"{label}: expected {expected!r}, found {found!r}")


def assert_true(value: bool, label: str) -> None:
    if not value:
        fail(label)


def timeout_at(seconds: float = TIMEOUT) -> float:
    return time.monotonic() + seconds


def remaining(deadline: float) -> float:
    value = deadline - time.monotonic()
    if value <= 0:
        raise TimeoutError("deadline expired")
    return value


def recv_or_none(
    receiver: Callable[..., dict[str, Any]],
    timeout: float = 0.05,
) -> dict[str, Any] | None:
    try:
        return receiver(timeout=timeout)
    except Empty:
        return None


def start_kernel() -> tuple[KernelManager, Any]:
    manager = KernelManager(kernel_name="rum-dev")
    manager.start_kernel(env=os.environ.copy())
    client = manager.client()
    client.start_channels()
    client.wait_for_ready(timeout=TIMEOUT)
    return manager, client


def stop_client(client: Any | None) -> None:
    if client is not None:
        client.stop_channels()


def shutdown_kernel(manager: KernelManager | None, now: bool = False) -> None:
    if manager is not None and manager.has_kernel:
        manager.shutdown_kernel(now=now)


def execute_collect(
    client: Any,
    code: str,
    *,
    stdin_replies: list[str] | None = None,
    allow_stdin: bool = True,
    timeout: float = TIMEOUT,
) -> ExecuteResult:
    deadline = timeout_at(timeout)
    replies = list(stdin_replies or [])
    result = ExecuteResult(code=code)
    msg_id = client.execute(code, allow_stdin=allow_stdin)
    idle_seen = False

    while True:
        made_progress = False

        stdin_msg = recv_or_none(client.get_stdin_msg)
        if stdin_msg is not None:
            made_progress = True
            result.stdin.append(stdin_msg)
            msg_type = stdin_msg["header"]["msg_type"]
            if msg_type != "input_request":
                fail(f"unexpected stdin message type {msg_type!r}")
            if not replies:
                fail(f"stdin request for {code!r} had no prepared reply")
            client.input(replies.pop(0))

        iopub_msg = recv_or_none(client.get_iopub_msg)
        if iopub_msg is not None:
            made_progress = True
            if iopub_msg.get("parent_header", {}).get("msg_id") == msg_id:
                result.iopub.append(iopub_msg)
                msg_type = iopub_msg["header"]["msg_type"]
                content = iopub_msg.get("content", {})
                if msg_type == "execute_result":
                    result.result_text = content.get("data", {}).get("text/plain")
                if msg_type == "status" and content.get("execution_state") == "idle":
                    idle_seen = True

        shell_msg = recv_or_none(client.get_shell_msg)
        if shell_msg is not None:
            made_progress = True
            if shell_msg.get("parent_header", {}).get("msg_id") == msg_id:
                result.shell = shell_msg

        if idle_seen and result.shell is not None:
            break

        if replies and result.shell is not None:
            fail("shell reply arrived before all prepared stdin replies were used")

        if not made_progress:
            time.sleep(min(0.02, remaining(deadline)))

    if replies:
        fail(f"{len(replies)} prepared stdin replies were not requested")

    return result


def shell_reply_for(client: Any, msg_id: str, label: str, timeout: float = TIMEOUT) -> dict[str, Any]:
    deadline = timeout_at(timeout)
    while True:
        msg = recv_or_none(client.get_shell_msg)
        if msg is not None and msg.get("parent_header", {}).get("msg_id") == msg_id:
            return msg
        time.sleep(min(0.02, remaining(deadline)))


def control_reply_for(
    client: Any, msg_id: str, label: str, timeout: float = TIMEOUT
) -> dict[str, Any]:
    deadline = timeout_at(timeout)
    while True:
        msg = recv_or_none(client.get_control_msg)
        if msg is not None and msg.get("parent_header", {}).get("msg_id") == msg_id:
            return msg
        time.sleep(min(0.02, remaining(deadline)))


def assert_shell_status(msg: dict[str, Any], expected: str, label: str) -> dict[str, Any]:
    content = msg.get("content", {})
    assert_equal(content.get("status"), expected, f"{label} shell status")
    return content


def send_shell_message(client: Any, msg_type: str, content: dict[str, Any]) -> str:
    msg = client.session.msg(msg_type, content)
    client.shell_channel.send(msg)
    return msg["header"]["msg_id"]


def send_control_message(client: Any, msg_type: str, content: dict[str, Any]) -> str:
    msg = client.session.msg(msg_type, content)
    client.control_channel.send(msg)
    return msg["header"]["msg_id"]


def assert_ok_execute(
    result: ExecuteResult,
    *,
    result_text: str | None,
    execution_count: int,
) -> None:
    assert_equal(result.shell_status, "ok", f"{result.code} shell status")
    assert_equal(result.execution_count, execution_count, f"{result.code} execution count")
    if result_text is not None:
        assert_equal(result.result_text, result_text, f"{result.code} text/plain result")
    assert_true(
        result.iopub_types[:2] == ["status", "execute_input"],
        f"{result.code} should start with busy status and execute_input",
    )
    assert_true(
        result.iopub_types[-1:] == ["status"],
        f"{result.code} should end with idle status",
    )


def lifecycle_smoke() -> None:
    manager: KernelManager | None = None
    client: Any | None = None
    try:
        manager, client = start_kernel()
        first = execute_collect(client, "1+1")
        assert_ok_execute(first, result_text="2", execution_count=1)
        print("lifecycle.first_execute=ok")

        stop_client(client)
        client = None
        manager.restart_kernel(now=False)
        client = manager.client()
        client.start_channels()
        client.wait_for_ready(timeout=TIMEOUT)

        after_restart = execute_collect(client, "2+2")
        assert_ok_execute(after_restart, result_text="4", execution_count=1)
        print("lifecycle.restart=ok")

        stop_client(client)
        client = None
        shutdown_kernel(manager, now=False)
        assert_true(not manager.has_kernel, "kernel should be stopped after shutdown")
        print("lifecycle.shutdown=ok")
    finally:
        stop_client(client)
        shutdown_kernel(manager, now=True)


def stdin_smoke() -> None:
    manager: KernelManager | None = None
    client: Any | None = None
    try:
        manager, client = start_kernel()

        single = execute_collect(
            client,
            'InputString["name: "]',
            stdin_replies=["Alice"],
        )
        assert_ok_execute(single, result_text='"Alice"', execution_count=1)
        assert_equal(single.stdin_prompts, ["name: "], "single stdin prompt")
        print("stdin.single_resume=ok")

        multi = execute_collect(
            client,
            'List[InputString["first: "], InputString["second: "]]',
            stdin_replies=["left", "right"],
        )
        assert_ok_execute(multi, result_text='List["left", "right"]', execution_count=2)
        assert_equal(multi.stdin_prompts, ["first: ", "second: "], "multi stdin prompts")
        print("stdin.multi_resume=ok")

        disallowed = execute_collect(
            client,
            'InputString["blocked: "]',
            allow_stdin=False,
        )
        assert_equal(disallowed.shell_status, "error", "disallowed stdin shell status")
        assert_equal(disallowed.stdin_prompts, [], "disallowed stdin prompts")
        assert_equal(disallowed.execution_count, 3, "disallowed stdin execution count")
        print("stdin.disallowed=ok")
    finally:
        stop_client(client)
        shutdown_kernel(manager, now=True)


def protocol_smoke() -> None:
    manager: KernelManager | None = None
    client: Any | None = None
    try:
        manager, client = start_kernel()

        defined = execute_collect(client, "localName = 42")
        assert_ok_execute(defined, result_text="42", execution_count=1)

        complete_id = client.complete("loc", cursor_pos=3)
        complete = assert_shell_status(
            shell_reply_for(client, complete_id, "complete"),
            "ok",
            "complete",
        )
        assert_true("localName" in complete.get("matches", []), "complete should return localName")
        assert_equal(complete.get("cursor_start"), 0, "complete cursor_start")
        assert_equal(complete.get("cursor_end"), 3, "complete cursor_end")
        print("protocol.complete=ok")

        inspect_id = client.inspect("localName", cursor_pos=len("localName"), detail_level=0)
        inspect = assert_shell_status(
            shell_reply_for(client, inspect_id, "inspect"),
            "ok",
            "inspect",
        )
        assert_equal(inspect.get("found"), True, "inspect found")
        assert_true(
            "text/plain" in inspect.get("data", {}),
            "inspect should include text/plain data",
        )
        assert_equal(
            inspect.get("metadata", {}).get("bundle_kind"),
            "inspect",
            "inspect metadata bundle_kind",
        )
        print("protocol.inspect=ok")

        incomplete_id = client.is_complete("f[\n  1")
        incomplete = shell_reply_for(client, incomplete_id, "is_complete incomplete")
        incomplete_content = incomplete.get("content", {})
        assert_equal(incomplete_content.get("status"), "incomplete", "is_complete incomplete status")
        assert_equal(incomplete_content.get("indent"), "    ", "is_complete incomplete indent")

        invalid_id = client.is_complete("f[1 2]")
        invalid = shell_reply_for(client, invalid_id, "is_complete invalid")
        assert_equal(
            invalid.get("content", {}).get("status"),
            "invalid",
            "is_complete invalid status",
        )
        print("protocol.is_complete=ok")

        history_id = client.history()
        history = assert_shell_status(
            shell_reply_for(client, history_id, "history"),
            "ok",
            "history",
        )
        assert_equal(history.get("history"), [], "history should be empty")
        print("protocol.history=ok")

        prints_and_messages = execute_collect(
            client,
            "Print[1]; Message[General::stop]; 2",
        )
        assert_ok_execute(prints_and_messages, result_text="2", execution_count=2)
        assert_true("1" in prints_and_messages.stdout_text, "Print should appear on stdout")
        assert_true(
            "General::stop" in prints_and_messages.stderr_text,
            "Message should appear on stderr",
        )
        print("protocol.streams=ok")

        execute_collect(client, "f[x_Integer] := g[x]")
        execute_collect(client, "g[x_Integer] := f[x]")
        failure = execute_collect(client, "f[1]")
        assert_equal(failure.shell_status, "error", "failure shell status")
        assert_true(
            "EvaluationCycleDetected" in failure.error_names,
            "failure should emit EvaluationCycleDetected on IOPub",
        )
        assert_true(
            "General::cycle" in failure.stderr_text,
            "failure should emit runtime message on stderr",
        )
        print("protocol.failure=ok")
    finally:
        stop_client(client)
        shutdown_kernel(manager, now=True)


def unsupported_smoke() -> None:
    manager: KernelManager | None = None
    client: Any | None = None
    try:
        manager, client = start_kernel()

        comm_id = client.comm_info()
        comm = assert_shell_status(
            shell_reply_for(client, comm_id, "comm_info"),
            "ok",
            "comm_info",
        )
        assert_equal(comm.get("comms"), {}, "comm_info should report no comms")
        print("unsupported.comm_info=ok")

        send_shell_message(
            client,
            "comm_open",
            {"comm_id": "rum-smoke-comm", "target_name": "rum.missing", "data": {}},
        )
        send_shell_message(
            client,
            "comm_msg",
            {"comm_id": "rum-smoke-comm", "data": {"value": 1}},
        )
        send_shell_message(
            client,
            "comm_close",
            {"comm_id": "rum-smoke-comm", "data": {}},
        )
        after_comm = execute_collect(client, "7+1")
        assert_ok_execute(after_comm, result_text="8", execution_count=1)
        print("unsupported.comm_messages=ok")

        debug_id = send_control_message(
            client,
            "debug_request",
            {
                "seq": 11,
                "type": "request",
                "command": "initialize",
                "arguments": {},
            },
        )
        debug = control_reply_for(client, debug_id, "debug_request")
        assert_equal(debug["header"]["msg_type"], "debug_reply", "debug reply type")
        debug_content = debug.get("content", {})
        assert_equal(debug_content.get("success"), False, "debug unsupported success flag")
        assert_equal(debug_content.get("command"), "initialize", "debug unsupported command")
        assert_true(
            "debug protocol" in debug_content.get("message", ""),
            "debug unsupported message",
        )
        after_debug = execute_collect(client, "8+1")
        assert_ok_execute(after_debug, result_text="9", execution_count=2)
        print("unsupported.debug=ok")

        send_shell_message(client, "rum_unknown_probe", {})
        after_unknown = execute_collect(client, "9+1")
        assert_ok_execute(after_unknown, result_text="10", execution_count=3)
        print("unsupported.unknown=ok")
    finally:
        stop_client(client)
        shutdown_kernel(manager, now=True)


def main(argv: list[str]) -> int:
    command = argv[1] if len(argv) > 1 else "all"
    if command in {"-h", "--help"}:
        print(
            "Usage: scripts/rum-jupyter-kernel-smoke.py "
            "[all|lifecycle|stdin|protocol|unsupported]"
        )
        return 0

    if command == "lifecycle":
        lifecycle_smoke()
    elif command == "stdin":
        stdin_smoke()
    elif command == "protocol":
        protocol_smoke()
    elif command == "unsupported":
        unsupported_smoke()
    elif command == "all":
        lifecycle_smoke()
        stdin_smoke()
        protocol_smoke()
        unsupported_smoke()
    else:
        print(
            "Usage: scripts/rum-jupyter-kernel-smoke.py "
            "[all|lifecycle|stdin|protocol|unsupported]",
            file=sys.stderr,
        )
        return 2

    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main(sys.argv))
    except Exception as error:
        print(f"kernel_smoke=error: {error}", file=sys.stderr)
        raise
