#!/usr/bin/env python3
import json
import sys
import os


def render_cell(cell):
    if isinstance(cell, bool):
        return "🌳" if cell else "🥀"
    elif isinstance(cell, float) or isinstance(cell, int):
        if cell > 5.0:
            return "🌱"
        else:
            return "."
    else:
        return cell


num_rows = 50
num_cols = 50

world = [[0.0 for _ in range(num_cols)] for _ in range(num_rows)]

if os.path.exists("world.json"):
    with open("world.json", "r") as f:
        world = json.load(f)

if sys.argv[1] == "observe":
    pass
elif sys.argv[1] == "set_energy":
    energy = json.loads(sys.argv[2])
    world[int(energy["x"])][int(energy["y"])] = energy["energy"]
elif sys.argv[1] == "set_life":
    life = json.loads(sys.argv[2])
    world[int(life["x"])][int(life["y"])] = life["life"]
elif sys.argv[1] == "set_cell":
    cell = json.loads(sys.argv[2])
    world[int(cell["x"])][int(cell["y"])] = cell["value"]

os.system("clear")

for row in world:
    for cell in row:
        print(render_cell(cell), end=" ")
    print()

with open("world.json", "w") as f:
    json.dump(world, f)
