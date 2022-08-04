"""
Write IPA data for testing
"""
import argparse
import random
from pathlib import Path
import sys

def get_args(args):

    parser = argparse.ArgumentParser(
        description = "Generate random IPA input data for MP-SPDZ")
    parser.add_argument("numrows_power", type = int, default = 12,
                        nargs = '?',
                        help = "Power of 2 of # of inputs")
    return parser.parse_args(args)

def process(numrows_power: int,
            modulus: int = 2 ** 31,
            mkmod: int = 2 ** 8,
            breakdown_keys: int = 4):
    head = [[3,1,11,0],
            [3,0,0,1],
            [2,1,7,0],
            [2,0,0,2],
            [4,0,0,3],
            [4,1,8,0],
            [4,1,6,0]
            ]
    numrows = 2 ** numrows_power
    yield from head
    yield from ([0,0,0,0] for _ in range(len(head), numrows))

def main():

    args = get_args(sys.argv[1: ])

    player_data = Path('Player-Data')
    player_data.mkdir(parents = True, exist_ok = True)

    with open(player_data / 'Input-P0-0', 'w') as fil:
        fil.write('\n'.join((' '.join(map(str,_)) for _ in process(args.numrows_power))))
        

main()

