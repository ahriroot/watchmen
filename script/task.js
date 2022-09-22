const sleep = (ms) => new Promise(resolve => setTimeout(resolve, ms))

const main = async () => {
    let count = 0
    while (true) {
        count++;
        console.log(`Result from nodejs task: ${count}`)
        if (count >= 5) {
            break;
        }
        await sleep(1000)
    }
}

main()
