#!/usr/bin/env python3
"""Read-only ELF function/layout audit; never execute the measured binaries.

Normalized equality removes relocatable address text and LLVM numeric suffixes.
It is a diagnostic, not proof of program equivalence or performance causation.
"""
import argparse
import hashlib
import json
import re
import struct
import subprocess
from pathlib import Path

NEED = (
    "::eval_sort", "::sort_normal_expression", "::compare_structural_order",
    "part_assignment::", "::replace_exact_path", "::compare_expr_canonical",
    "::normalize_call_args_by_attributes", "::normalize_call_args_shared",
    "::replace_owned", "::replace_call_children_owned",
)


def digest(data):
    return hashlib.sha256(data).hexdigest()


def elf_text(data):
    if data[:6] != b"\x7fELF\x02\x01":
        raise ValueError("Expected 64-bit little-endian ELF")
    offset = struct.unpack_from("<Q", data, 40)[0]
    stride, count, names_index = struct.unpack_from("<HHH", data, 58)
    headers = [struct.unpack_from("<IIQQQQIIQQ", data, offset + i * stride)
               for i in range(count)]
    names_header = headers[names_index]
    names = data[names_header[4]:names_header[4] + names_header[5]]
    for header in headers:
        if names[header[0]:].split(b"\0", 1)[0] == b".text":
            return header[3], header[4], header[5]
    raise ValueError("Missing .text section")


def normalize(output):
    lines = []
    for line in output.splitlines():
        match = re.match(r"\s*[0-9a-f]+:\s*(.*)", line)
        if not match:
            continue
        instruction = match[1]
        instruction = re.sub(r"\b[0-9a-f]+ <", "<", instruction)
        instruction = re.sub(r"0x[0-9a-f]+\(%rip\)", "<rip>(%rip)", instruction)
        instruction = re.sub(r"::h[0-9a-f]{16}", "", instruction)
        instruction = re.sub(r"\.llvm\.\d+", ".llvm.ID", instruction)
        instruction = re.sub(r"\s+# [0-9a-f]+", " #", instruction)
        lines.append(instruction)
    return lines


def inspect(path):
    data = Path(path).read_bytes()
    text_addr, text_offset, text_size = elf_text(data)
    symbols = subprocess.check_output(["nm", "-S", "-C", str(path)], text=True)
    functions = {}
    for line in symbols.splitlines():
        parts = line.split(None, 3)
        if (len(parts) != 4 or parts[2] not in ("t", "T")
                or not any(name in parts[3] for name in NEED)):
            continue
        addr, size = int(parts[0], 16), int(parts[1], 16)
        body_offset = text_offset + addr - text_addr
        disassembly = subprocess.check_output([
            "objdump", "-d", "-C", "--no-show-raw-insn",
            "--start-address=" + str(addr), "--stop-address=" + str(addr + size),
            str(path),
        ], text=True)
        instructions = normalize(disassembly)
        if parts[3] in functions:
            raise ValueError("Ambiguous selected symbol: " + parts[3])
        functions[parts[3]] = {
            "address": hex(addr), "size": size,
            "raw_sha256": digest(data[body_offset:body_offset + size]),
            "normalized_sha256": digest("\n".join(instructions).encode()),
            "normalized_instructions": instructions,
        }
    return {
        "binary": str(path), "binary_sha256": digest(data),
        "text_address": hex(text_addr), "text_size": text_size,
        "text_sha256": digest(data[text_offset:text_offset + text_size]),
        "functions": functions,
    }


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--base", required=True, type=Path)
    parser.add_argument("--candidate", required=True, type=Path)
    parser.add_argument("--independent", required=True, type=Path)
    parser.add_argument("--output", required=True, type=Path)
    args = parser.parse_args()
    results = {name: inspect(getattr(args, name))
               for name in ("base", "candidate", "independent")}
    comparisons = []
    for name, base in results["base"]["functions"].items():
        candidate = results["candidate"]["functions"].get(name)
        independent = results["independent"]["functions"].get(name)
        if candidate is None or independent is None:
            continue
        comparisons.append({
            "name": name,
            "sizes": [base["size"], candidate["size"], independent["size"]],
            "base_candidate_raw_equal": base["raw_sha256"] == candidate["raw_sha256"],
            "base_candidate_normalized_equal": base["normalized_sha256"] == candidate["normalized_sha256"],
            "candidates_raw_equal": candidate["raw_sha256"] == independent["raw_sha256"],
            "candidates_normalized_equal": candidate["normalized_sha256"] == independent["normalized_sha256"],
        })
    result = {
        "scope": "Selected function bytes and address-normalized disassembly only",
        "limitation": "Address normalization is lossy; equality does not prove identical data, transitive callees, execution behavior, or a cause of timing differences.",
        "normalization": "Remove instruction addresses, symbolic-target absolute addresses, RIP-relative displacements, Rust hash text, and LLVM numeric suffixes; preserve opcodes, registers, literal constants, named targets and within-function target offsets.",
        "commands": ["nm -S -C <binary>", "objdump -d -C --no-show-raw-insn --start-address=<decimal function start> --stop-address=<decimal function end> <binary>"],
        "tool_versions": {tool: subprocess.check_output([tool, "--version"], text=True).splitlines()[0]
                          for tool in ("nm", "objdump")},
        "binaries": results, "comparisons": comparisons,
    }
    args.output.write_text(json.dumps(result, indent=2) + "\n", encoding="utf-8")
    print(json.dumps({"output": str(args.output), "function_count": len(comparisons),
                      "changed_normalized": [row["name"] for row in comparisons
                                             if not row["base_candidate_normalized_equal"]]}))


if __name__ == "__main__":
    main()
