import os
import subprocess


def watchmen():
    argv = os.sys.argv
    current_path = os.path.dirname(os.path.abspath(__file__))
    path = "target/release/watchmen"
    bin_path = os.path.join(current_path, path)
    if os.path.exists(bin_path):
        subprocess.run([bin_path, *argv[1:]])
    else:
        raise Exception("watchmen binary not found")


def watchmend():
    argv = os.sys.argv
    current_path = os.path.dirname(os.path.abspath(__file__))
    path = "target/release/watchmend"
    bin_path = os.path.join(current_path, path)
    if os.path.exists(bin_path):
        subprocess.run([bin_path, *argv[1:]])
    else:
        raise Exception("watchmend binary not found")
