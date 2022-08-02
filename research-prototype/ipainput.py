<<<<<<< HEAD
import random
import argparse
from pathlib import Path

parser = argparse.ArgumentParser(description="Generate random IPA input data for MP-SPDZ")
=======
import argparse
import random
from pathlib import Path

parser = argparse.ArgumentParser(
    description="Generate random IPA input data for MP-SPDZ"
)
>>>>>>> bfd8bcc6c5400f492ab23aa8d114a2ad108aa4df
parser.add_argument("numrows_power", type=int, help="Generate 2^n inputs")

args = parser.parse_args()

<<<<<<< HEAD
numrows = 2 ** args.numrows_power
=======
numrows = 2**args.numrows_power
>>>>>>> bfd8bcc6c5400f492ab23aa8d114a2ad108aa4df
approx_rows_per_mk = 10

modulus = 2**64
mkmod = 2**31
datamod = 2**8
breakdown_keys = 4

player_data = Path("Player-Data")
player_data.mkdir(parents=True, exist_ok=True)

with open(player_data / "Input-P0-0", "w") as f:
    for i in range(numrows):
<<<<<<< HEAD
        mk = random.randint(0, numrows//approx_rows_per_mk)
        is_trigger = random.randint(0, 1)

        if(is_trigger):
            value, bk = random.randint(0, datamod-1), 0
        else:
            value, bk = 0, random.randint(0, breakdown_keys-1)
=======
        mk = random.randint(0, numrows // approx_rows_per_mk)
        is_trigger = random.randint(0, 1)

        if is_trigger:
            value, bk = random.randint(0, datamod - 1), -1
        else:
            value, bk = 0, random.randint(0, breakdown_keys - 1)
>>>>>>> bfd8bcc6c5400f492ab23aa8d114a2ad108aa4df
        f.write(f"{mk}\n{is_trigger}\n{value}\n{bk}\n")  # data = value

print(f"wrote {numrows} rows")
