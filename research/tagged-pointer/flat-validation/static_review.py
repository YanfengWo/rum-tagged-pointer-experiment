"""Static audit; no measured binary execution."""
import argparse
import hashlib
import json
from pathlib import Path
import re
import struct
import subprocess
import sys
sys.dont_write_bytecode = True
from got_targets import normalize, relocation_targets

NEED = ("::eval_string_trim", "::trim_string_value", "::trim_whitespace_chars",
        "::trim_string_patterns", "::trim_literal", "::longest_matching_literal_length",
        "::parse_string_boundary_spec", "::flatten_flat_call_args",
        "::normalize_call_args_by_attributes", "::normalize_call_args_shared")


def sha(data):
    return hashlib.sha256(data).hexdigest()


def command(args):
    return subprocess.check_output(args, text=True, timeout=120)


def canonical(name):
    return re.sub(r"\.llvm\.\d+", ".llvm.ID", re.sub(r"::h[0-9a-f]{16}", "", name))


def text_section(data):
    assert data[:6] == b"\x7fELF\x02\x01"
    offset = struct.unpack_from("<Q", data, 40)[0]
    stride, count, index = struct.unpack_from("<HHH", data, 58)
    headers = [struct.unpack_from("<IIQQQQIIQQ", data, offset + i * stride) for i in range(count)]
    header = headers[index]
    names = data[header[4]:header[4] + header[5]]
    return next(h for h in headers if names[h[0]:].split(b"\0", 1)[0] == b".text")


def inspect(item, out):
    path, mode = Path(item["binary"]), item["mode"]
    data = path.read_bytes()
    assert sha(data) == item["binary_sha256"]
    section = text_section(data)
    symbols = command(["nm", "-S", "-C", str(path)])
    table = {}
    for line in symbols.splitlines():
        parts = line.split(None, 3)
        if len(parts) == 4 and parts[2] in {"t", "T"}:
            address, size = int(parts[0], 16), int(parts[1], 16)
            if section[3] <= address < section[3] + section[5]:
                table.setdefault(address, []).append((size, parts[3]))
    got_targets = relocation_targets(data, table)
    roots = {address for address, entries in table.items()
             if any(any(part in name for part in NEED) for _, name in entries)}
    assert roots, mode
    functions, logs = {}, []

    def body(address, role):
        size, name = max(table[address])
        assert size > 0
        offset = section[4] + address - section[3]
        assembly = command(["objdump", "-d", "-C", "--no-show-raw-insn",
                            "--start-address=" + str(address), "--stop-address=" + str(address + size), str(path)])
        logs.append(assembly)
        instructions = normalize(assembly, got_targets)
        direct, indirect = [], []
        for line in assembly.splitlines():
            match = re.search(r"\b(?:callq?|jmpq?)\s+([0-9a-f]+)\s+<(.+)>", line)
            if match and not address <= int(match[1], 16) < address + size:
                direct.append({"address": hex(int(match[1], 16)), "symbol": canonical(match[2])})
            elif re.search(r"\b(?:callq?|jmpq?)\s+\*", line):
                indirect.append(normalize(line, got_targets)[0])
        key = canonical(name)
        if key in functions:
            old = functions.pop(key)
            functions[key + " @" + old["address"]] = old
            key += " @" + hex(address)
        elif any(name.startswith(key + " @") for name in functions):
            key += " @" + hex(address)
        functions[key] = {"role": role, "address": hex(address), "alignment_mod_64": address % 64,
                          "size": size,
                          "raw_sha256": sha(data[offset:offset + size]),
                          "normalized_sha256": sha("\n".join(instructions).encode()),
                          "instruction_count": len(instructions), "direct_control_targets": direct,
                          "indirect_control_transfers": indirect,
                          "normalized_instructions": instructions}
        return {int(target["address"], 16) for target in direct}

    callees = set()
    for address in sorted(roots):
        callees.update(body(address, "selected-root"))
    for address in sorted(callees - roots):
        if address in table and max(table[address])[0] <= 30000:
            body(address, "one-hop-direct-callee")
    log = "\n".join(logs)
    (out / f"{mode}-static-disassembly.log").write_text(log, encoding="utf-8")
    return {"binary": str(path), "binary_sha256": sha(data), "text_address": hex(section[3]),
            "text_size": section[5], "text_sha256": sha(data[section[4]:section[4] + section[5]]),
            "disassembly_log_sha256": sha(log.encode()), "resolved_relocation_targets": len(got_targets), "functions": functions}


def source_audit(source):
    excerpts = {
        "scripts/core-language-microbench.sh": [(291, 299), (322, 339), (388, 405), (617, 653)],
        "crates/rum-stdlib/src/strings/patterns/search.rs": [(164, 196)],
        "crates/rum-stdlib/src/strings/patterns/matcher/api.rs": [(247, 256)],
        "crates/rum-stdlib/src/strings/patterns/matcher/split_trim.rs": [(157, 184), (214, 247)],
        "crates/rum-runtime/src/runtime/call_preparation.rs": [(338, 356)],
        "crates/rum-runtime/src/runtime/builtin_catalog/stdlib_attributes/specs_s_z.rs": [(59, 59)],
    }
    result = {}
    for name, spans in excerpts.items():
        data = (source / name).read_bytes()
        lines = data.decode("utf-8").splitlines()
        result[name] = {"sha256": sha(data), "excerpts": [
            {"first_line": a, "last_line": b, "text": "\n".join(lines[a - 1:b])} for a, b in spans]}
    assert 'Spec::new("StringTrim", ATTR_PROTECTED)' in (source / next(n for n in excerpts if n.endswith("specs_s_z.rs"))).read_text()
    callers = []
    for path in (source / "crates").rglob("*.rs"):
        for number, line in enumerate(path.read_text(encoding="utf-8").splitlines(), 1):
            if "flatten_flat_call_args(" in line:
                callers.append({"file": path.relative_to(source).as_posix(), "line": number, "text": line.strip()})
    assert len(callers) == 2, callers
    return {"files": result, "flat_helper_definition_and_calls": callers,
            "observations": [
                "Six labels trim preconstructed 4000/32000-character strings with literal-list, single-literal or whitespace boundaries.",
                "StringTrim is Protected, without Flat; trim_string_value collects chars then selects whitespace or pattern/literal helpers.",
                "The sole explicit Flat helper caller is guarded by Attribute::Flat; StringTrim/trim helpers do not directly call it.",
                "Not a dynamic trace: evaluator, allocation/drop and wrapper still matter; n*scale/k+1 are outside the timed Do."]}


def main(out):
    builds = json.loads((out / "builds.json").read_text(encoding="utf-8"))
    assert builds["status"] == "PASS" and not builds["instrumentation_present"]
    binaries = {item["mode"]: inspect(item, out) for item in builds["variants"]}
    comparisons = []
    for left, right in [("base", "wide"), ("base", "flat"), ("wide", "combined"), ("flat", "combined")]:
        a, b = binaries[left]["functions"], binaries[right]["functions"]
        rows = []
        for name in sorted(set(a) & set(b)):
            if " @0x" in name:
                continue
            rows.append({"symbol": name, "role": a[name]["role"],
                         "addresses": [a[name]["address"], b[name]["address"]],
                         "sizes": [a[name]["size"], b[name]["size"]],
                         "raw_equal": a[name]["raw_sha256"] == b[name]["raw_sha256"],
                         "normalized_equal": a[name]["normalized_sha256"] == b[name]["normalized_sha256"]})
        comparisons.append({"left": left, "right": right, "common": rows,
                            "only_left": sorted(set(a) - set(b)), "only_right": sorted(set(b) - set(a))})
    result = {"status": "PASS", "scope": "Selected roots and one-hop direct callees; no execution/timings.",
              "normalization": "Resolve named local/dynamic GOT relocation targets; remove addresses, RIP displacements, Rust hashes and LLVM suffixes. Unresolved GOT offsets remain. Preserve instructions, registers and constants.",
              "limitations": ["Address normalization is lossy and may hide changed data targets or allocation/dispatch destinations.",
                              "One-hop direct callees are not a transitive closure. Indirect calls and inlined code are not proven equivalent.",
                              "Identical selected normalized functions do not imply identical whole-program execution, code/data layout, caches or timing.",
                              "Address/size/instruction differences are compilation observations, not established causes of measured StringTrim regressions."],
              "tool_versions": {tool: command([tool, "--version"]).splitlines()[0] for tool in ["nm", "objdump"]},
              "builds_sha256": sha((out / "builds.json").read_bytes()),
              "source": source_audit(Path(builds["source_root"])),
              "binaries": binaries, "comparisons": comparisons}
    path = out / "static-comparison.json"
    path.write_text(json.dumps(result, indent=2) + "\n", encoding="utf-8")
    print(json.dumps({"status": "PASS", "output": str(path),
                      "root_symbols": {mode: [name for name, row in data["functions"].items() if row["role"] == "selected-root"]
                                       for mode, data in binaries.items()},
                      "changed_normalized": {row["left"] + "/" + row["right"]: [v["symbol"] for v in row["common"] if not v["normalized_equal"]]
                                             for row in comparisons}}, indent=2))


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("evidence", type=Path)
    main(parser.parse_args().evidence.resolve())
