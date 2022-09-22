import time


def main():
    # while 循环 10 次
    count = 0
    while True:
        count += 1
        print(f'Result from python task: {count}')
        if count >= 100:
            break
        time.sleep(1)


if __name__ == '__main__':
    main()
