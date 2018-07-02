#! /usr/bin/env python3.6

import sys
import time
import numpy as np
import gc


SLEEP = 0

def read(ndata, path):
    data = np.empty(ndata, dtype=np.float64)
    with open(path) as fp:
        i = 0
        for line in fp:
            try:
                val = float(line)
            except ValueError:
                continue
            data[i] = val
            i += 1
            if i >= ndata:
                break

    # User asked for more data than available; truncate our initial
    # estimate
    if ndata > i:
        data.resize(ndata)

    return data


def main(ndata, path):
    data = read(ndata, path)

    time.sleep(SLEEP)
    print("Data size =", len(data)*8 / 1024 / 1024, "MB")
    time.sleep(SLEEP)
    gc.collect()
    time.sleep(SLEEP)
    print('Mean = {:.10e}'.format(data.mean()))
    t = time.process_time()
    m = np.median(data)
    t = time.process_time() - t
    print('Median = {:.10e}'.format(m))
    print("Duration =", t, "seconds")


if __name__ == '__main__':
    if len(sys.argv) != 3:
        sys.exit("Usage: {} <ndata> <file.dat>".format(sys.argv[0]))
    main(int(sys.argv[1]), sys.argv[2])
