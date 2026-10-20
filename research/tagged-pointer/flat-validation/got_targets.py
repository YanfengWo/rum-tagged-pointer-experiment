"""Resolve ELF64 GOT relocation names for a lossy static instruction comparison."""
import re
import struct


def canonical(name):
    return re.sub(r"\.llvm\.\d+", ".llvm.ID", re.sub(r"::h[0-9a-f]{16}", "", name))


def relocation_targets(data, table):
    assert data[:6] == b"\x7fELF\x02\x01"
    offset = struct.unpack_from("<Q", data, 40)[0]
    stride, count = struct.unpack_from("<HH", data, 58)
    headers = [struct.unpack_from("<IIQQQQIIQQ", data, offset + i * stride) for i in range(count)]
    result = {}
    for header in headers:
        if header[1] != 4:  # SHT_RELA
            continue
        symbols = headers[header[6]]
        strings = headers[symbols[6]] if symbols[1] in {2, 11} else None
        for pos in range(header[4], header[4] + header[5], header[9] or 24):
            slot, info, addend = struct.unpack_from("<QQq", data, pos)
            kind, index = info & 0xffffffff, info >> 32
            if kind == 8 and addend in table:  # R_X86_64_RELATIVE
                result[slot] = "local:" + canonical(max(table[addend])[1])
            elif kind in {6, 7} and strings is not None:  # GLOB_DAT / JUMP_SLOT
                name = struct.unpack_from("<I", data, symbols[4] + index * (symbols[9] or 24))[0]
                begin = strings[4] + name
                result[slot] = "dynamic:" + data[begin:data.index(b"\0", begin)].decode("utf-8")
    return result


def normalize(disassembly, targets):
    lines = []
    for line in disassembly.splitlines():
        match = re.match(r"\s*[0-9a-f]+:\s*(.*)", line)
        if not match:
            continue
        instruction = match[1]
        got = re.search(r"#\s*([0-9a-f]+)\s+<(_GLOBAL_OFFSET_TABLE_\+0x[0-9a-f]+)>", instruction)
        if got and int(got[1], 16) in targets:
            instruction = instruction[:got.start()] + "# GOT[" + targets[int(got[1], 16)] + "]"
        instruction = re.sub(r"\b[0-9a-f]+ <", "<", instruction)
        instruction = re.sub(r"-?0x[0-9a-f]+\(%rip\)", "<rip>(%rip)", instruction)
        instruction = re.sub(r"\s+# [0-9a-f]+", " #", instruction)
        lines.append(canonical(instruction))
    return lines
