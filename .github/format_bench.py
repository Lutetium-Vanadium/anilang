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

def lower_to_unit(num, unit, to='ns'):
    if unit == to:
        return num

    unit_i = units.index(unit)
    to_i = units.index(to)

    return num * (unit_i - to_i) * 1000

benches = {
    'full-no_optimize': [0, 0],
    'full-optimize': [0, 0],
    'lexer': [0, 0],
    'parser': [0, 0],
    'lower-no_optimize': [0, 0],
    'evaluate-no_optimize': [0, 0],
    'lower-optimize': [0, 0],
    'evaluate-optimize': [0, 0],
}


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

    print(f'| {id} | {cur} {unit} |', end='')

    if change:
        previous = typical_estimate * (1 + change)

        (prev, prev_unit) = try_bump_unit(previous, orig_unit, unit)
        (diff, diff_unit) = try_bump_unit(previous - typical_estimate, orig_unit, unit)
        diff_percent = round(100 * change, 2)

        bench_type = id.split('/')[-1]
        benches[bench_type][0] += diff_percent
        benches[bench_type][1] += 1

        print(f' {prev} {prev_unit} | {format_signed(diff)} {diff_unit} | {format_signed(diff_percent)}% | {change_txt} |')
    else:
        print(' - | - | - | - |')

print('\nAverage diff%\n')
print('| benchmark | average diff% | change |')
print('| --------- | ------------- | ------ |')

for (bench_type, [total_diff_percent, count]) in benches.items():
    if count == 0:
        continue

    diff_percent = round(total_diff_percent / count, 2)

    change_txt = 'No Change'

    # Arbitrary value for noise threshold
    if diff_percent > 2.0:
        change_txt = 'Regressed'
    elif diff_percent < -2.0:
        change_txt = 'Improved'

    print(f'| {bench_type} | {format_signed(diff_percent)}% | {change_txt} |')
