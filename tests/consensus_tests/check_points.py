#!/usr/bin/env python3

import requests
import sys

from assertions import assert_http_ok

# Check that 'search' returns the same results on all peers
for i in range(2, len(sys.argv)):
    r = requests.post(
       f"http://127.0.0.1:{sys.argv[i]}/collections/{sys.argv[1]}/points/search", json={
           "vector": [0.2, 0.1, 0.9, 0.7],
           "top": 3,
       }
    )
    assert_http_ok(r)
    assert r.json()["result"][0]["id"] == 4
    assert r.json()["result"][1]["id"] == 1
    assert r.json()["result"][2]["id"] == 3
