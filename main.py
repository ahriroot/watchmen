import os
import subprocess


def watchmen():
    try:
        argv = os.sys.argv
        current_path = os.path.dirname(os.path.abspath(__file__))
        path = "target/release/watchmen"
        bin_path = os.path.join(current_path, path)
        if os.path.exists(bin_path):
            subprocess.run([bin_path, *argv[1:]])
        else:
            raise Exception("watchmen binary not found")
    except InterruptedError:
        print("Interrupted by user")
    except Exception as e:
        print(e)


def watchmend():
    try:
        argv = os.sys.argv
        current_path = os.path.dirname(os.path.abspath(__file__))
        path = "target/release/watchmend"
        bin_path = os.path.join(current_path, path)
        if os.path.exists(bin_path):
            subprocess.run([bin_path, *argv[1:]])
        else:
            raise Exception("watchmend binary not found")
    except InterruptedError:
        print("Interrupted by user")
    except Exception as e:
        print(e)
