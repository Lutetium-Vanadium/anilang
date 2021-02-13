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

def try_bump_unit(num, unit):
    unit_i = units.index(unit)

    while unit_i < len(units) - 1 and abs(num) > 1000:
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
    unit = json['unit']
    typical_estimate = json['typical']['estimate']
    cur, cur_unit = try_bump_unit(typical_estimate, unit)
    change_txt, change = format_change(json['change'])

    print(f'| {id} | {cur} {cur_unit} |', end='')

    if change:
        previous = typical_estimate * (1 + change)

        prev, prev_unit = try_bump_unit(previous, unit)
        diff, diff_unit = try_bump_unit(previous - typical_estimate, unit)
        diff_percent = round(100 * change, 2)

        print(f' {prev} {prev_unit} | {format_signed(diff)} {diff_unit} | {format_signed(diff_percent)}% | {change_txt} |')
    else:
        print(' - | - | - | - |')
