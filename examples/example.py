import time

import rust_py as r

rt = r.create_runtime()
c = r.create_client(rt)
it = 0
while True:
    time.sleep(2)
    print("Iter in python : ", it)
    it += 1
