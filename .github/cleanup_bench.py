import sys

COMPILE_DELIM = "Finished bench"

is_compiling = True
skip_start = True

for orig_line in sys.stdin:
    line = orig_line.strip()
    if is_compiling:
        if line[:len(COMPILE_DELIM)] == COMPILE_DELIM:
            is_compiling = False
        continue
    if skip_start and len(line) == 0:
        if len(line) == 0:
            continue
        else:
            skip_start = False
    
    if line.lower().find("running target/release/deps") >= 0:
        continue
    if line.lower().find("test") >= 0:
        continue
    
    # Do not add extra '\n'
    print(orig_line, end="")
