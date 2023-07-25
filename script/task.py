import sys
import time


def main():
    # print('Python task is running...')
    # while True:
    #     print('Python task are running...')
    #     data = sys.stdin.readline().rstrip()
    #     print(f'Result from python task: {data}')
    # count = 0
    # while True:
    #     count += 1
    #     print(f'Result from python task: {count}')
    #     if count >= 5:
    #         raise Exception('Python task error')
    #     time.sleep(1)
    count = 0
    while True:
        count += 1
        print(f'Result from python task: {count}')
        if count >= 100:
            break
        time.sleep(1)


if __name__ == '__main__':
    main()
