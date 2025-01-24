import sys
import re

out = open("perf.folded", "w")


def read_lines(inp):
    while True:
        line = inp.readline()
        if line == "":
            break
        yield line


def to_micros(dur: str) -> int:
    if dur.endswith("µs"):
        return int(float(dur[:-2]))
    if dur.endswith("ms"):
        return int(1000 * float(dur[:-2]))
    if dur.endswith("s"):
        return int(1000_000 * float(dur[:-1]))
    raise RuntimeError()


def handle_enter(stack):
    pass


def handle_leave(stack, dur: str):
    time = to_micros(dur)
    print(";".join(stack[1:]), time, file=out)


def dbg(val):
    print(val)
    return val


stacks = {}

info = re.compile(r"DEBUG - \(HART(\d+) at (\d+)\) perf: (.+)$")

lines = read_lines(sys.stdin)
lines = map(info.search, lines)
lines = filter(bool, lines)
lines = map(re.Match.groups, lines)
for hart, _timestamp, msg in lines:
    stack = stacks.setdefault(hart, [None])

    m = msg.split()

    if m[0] == "entering":
        stack.append(m[1])
        handle_enter(stack)
        continue
    assert m[1] == "took", m

    if stack[-1] != m[0]:
        stack.append(m[0])
        handle_enter(stack)

    handle_leave(stack, m[2])
    prev = stack.pop()
    assert prev == m[0]
