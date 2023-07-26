import sys
import time


def main():
    # print('Python task is running...')
    # while True:
    #     print('Python task are running...')
    #     data = sys.stdin.readline().rstrip()
    #     print(f'Result from python task: {data}')
    index = int(sys.argv[1]) if len(sys.argv) > 1 else 100
    count = 0
    while True:
        count += 1
        print(f'Result from python task: {count}')
        if count >= index:
            break
        time.sleep(1)


if __name__ == '__main__':
    main()
