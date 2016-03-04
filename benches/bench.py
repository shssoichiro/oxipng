#!/usr/bin/env python
import subprocess;
from time import time;
from sys import exit;
from os.path import isfile;

results = [];

filename = "bench.png"
if not isfile(filename):
    print("Invalid file");
    exit(1);
for opt_level in range(1, 7):
    start_time = time();
    command = subprocess.Popen(["oxipng", "-o", str(opt_level), "-q", "--stdout", filename], stdout=subprocess.PIPE);
    size = len(command.communicate()[0]);
    result_time = time()-start_time;
    results.append({"program": "oxipng", "opt_level": opt_level, "time": result_time, "size": size});

for result in results:
    print("%s -o%s: \n Time: %ss \n Size: %s bytes" % (result["program"], result["opt_level"], result["time"], result["size"]));
