#!/usr/bin/env python3

import requests
import sys

url_template = "https://api.chess.com/pub/player/{player}/games/{year}/{month}/pgn"

if len(sys.argv) < 4:
    print("Usage: <user> <yyyy> <mm>")
    sys.exit(2)

player = sys.argv[1]
year = sys.argv[2]
month = sys.argv[3]

myurl = url_template.format(year=year, month=month, player=player)
result = requests.get(myurl)
import pdb; pdb.set_trace()
print("fetched "+myurl)
print(result)
