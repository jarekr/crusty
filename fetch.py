#!/usr/bin/env python3

import requests

url_template = "https://api.chess.com/pub/player/jrudzinski/games/{year}/{month}/pgn"

result = requests.get(url_template.format(year=2020, month=10))
import pdb; pdb.set_trace()
print("all done")
