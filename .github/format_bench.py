import sys
from json import loads as parse_json

def format_change(change):
    if change == None:
        return ('-', None)
    
    return (
        'No change' if (a := change['change']) == 'NoChange' else a,
        change['mean']['estimate']
    )

def format_signed(num):
    if num < 0:
        return f'-{-num}'
    else:
        return f'+{num}'

units = ['ns', 'us', 'ms', 's']

def try_bump_unit(num, unit, max='s'):
    unit_i = units.index(unit)
    max_i = units.index(max)

    while unit_i < max_i and abs(num) > 1000:
        unit_i += 1
        num /= 1000

    return round(num, 2), units[unit_i]



print('| benchmark | current time | previous time | diff | diff% | change |')
print('| --------- | ------------ | ------------- | ---- | ----- | ------ |')

for line in sys.stdin:
    json = parse_json(line)
    if json['reason'] != 'benchmark-complete':
        continue

    id = json['id']
    orig_unit = json['unit']
    typical_estimate = json['typical']['estimate']
    cur, unit = try_bump_unit(typical_estimate, orig_unit)
    change_txt, change = format_change(json['change'])

    print(f'| {id} | {cur} {cur_unit} |', end='')

    if change:
        previous = typical_estimate * (1 + change)

        prev = try_bump_unit(previous, orig_unit, unit)[0]
        diff = try_bump_unit(previous - typical_estimate, orig_unit, unit)[0]
        diff_percent = round(100 * change, 2)

        print(f' {prev} {unit} | {format_signed(diff)} {unit} | {format_signed(diff_percent)}% | {change_txt} |')
    else:
        print(' - | - | - | - |')
