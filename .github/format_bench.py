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

print('| benchmark | current time | previous time | diff | diff% | change |')
print('| --------- | ------------ | ------------- | ---- | ----- | ------ |')

for line in sys.stdin:
    json = parse_json(line)
    if json['reason'] != 'benchmark-complete':
        continue

    id = json['id']
    unit = json['unit']
    typical_estimate = json['typical']['estimate']
    change_txt, change = format_change(json['change'])

    print(f'| {id} | {round(typical_estimate, 2)} {unit} |', end='')

    if change:
        previous = round(typical_estimate * (1 + change), 2)
        diff = round(previous - typical_estimate, 2)
        diff_percent = round(100 * change, 2)
        print(f' {previous} {unit} | {format_signed(diff)} {unit} | {format_signed(diff_percent)}% | {change_txt} |')
    else:
        print(' - | - | - | - |')
