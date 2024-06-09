#!/usr/bin/env python3
import argparse

parser = argparse.ArgumentParser()

parser.add_argument('--character_name', dest='character_name',
                    help='the character name')
parser.add_argument('message')

args = parser.parse_args()

print("%s: %s" % (args.character_name, args.message))
