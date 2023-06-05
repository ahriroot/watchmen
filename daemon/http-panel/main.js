import { createApp } from 'petite-vue';
import 'bootstrap/dist/css/bootstrap.css';

const request = async (command) => {
    let response = await fetch('http://127.0.0.1:1997/api', {
        method: 'POST',
        body: JSON.stringify(command),
        headers: {
            'Content-Type': 'application/json'
        }
    })
    let body = await response.json()
    if (body.code === 10000) {
        return body.data
    } else {
        throw new Error(`${body.msg} - ${body.data.String}`)
    }
}

createApp({
    dialog: false,
    modal: false,
    theme: {
        main: 'dark',
        second: 'secondary',
    },
    filter: '',
    tasks: [],
    task: {
        "id": 0,
        "name": "Default",
        "command": "python",
        "args": "-u,$HOME/watchmen/script/task.py",
        "dir": null,
        "env": {},
        "stdin": false,
        "stdout": "$HOME/watchmen/logs/stdout.log",
        "stderr": "",
        "created_at": 1685950437,
        "task_type": "async",
        "pid": null,
        "status": "added",
        "code": null
    },
    info: '',
    width: false,
    async init() {
        window.addEventListener('resize', () => {
            this.width = window.innerWidth
        })
        await this.getTasks()
    },
    async infoTask(t) {
        this.info = JSON.stringify(t, null, 4)
        this.modal = true
    },
    async getTasks() {
        request({ "command": { "List": null } }).then(data => {
            this.tasks = data.Status
        }).catch(err => {
            alert(err)
        })
    },
    async _req(opera, name) {
        request({ "command": { [opera]: name } }).then(async _ => {
            await this.getTasks()
        }).catch(err => {
            alert(err)
        })
    },
    async startTask(name) {
        await this._req('Start', name)
    },
    async stopTask(name) {
        await this._req('Stop', name)
    },
    async removeTask(name) {
        await this._req('Remove', name)
    },
    async addTask() {
        let dir = this.task.dir == null ? null : this.task.dir.trim()
        let stdout = this.task.stdout.trim()
        let stderr = this.task.stderr.trim()
        let id = parseInt(new Date().getTime() / 1000)
        let task_type = null
        if (this.task.task_type == 'async') {
            task_type = {
                "Async": {
                    "started_at": 0,
                    "stopped_at": 0
                }
            }
        } else if (this.task.task_type == 'scheduled') {
            task_type = {
                "Scheduled": {
                    "year": null,
                    "month": null,
                    "day": null,
                    "hour": null,
                    "minute": null,
                    "second": null,
                }
            }
        } else if (this.task.task_type == 'periodic') {
            task_type = {
                "Periodic": {
                    "started_after": 0,
                    "interval": 60,
                    "last_run": 0,
                }
            }
        }
        let cmd = {
            "command": {
                "Add": {
                    "id": id,
                    "name": this.task.name.trim(),
                    "command": this.task.command.trim(),
                    "args": this.task.args.split(',').map(v => v.trim()),
                    "dir": dir == "" ? null : dir,
                    "env": {},
                    "stdin": this.task.stdin ? this.task.stdin : null,
                    "stdout": stdout == "" ? null : stdout,
                    "stderr": stderr == "" ? null : stderr,
                    "created_at": id,
                    "task_type": task_type,
                    "pid": null,
                    "status": "added",
                    "code": null
                }
            }
        }
        await request(cmd)
        this.dialog = false
        await this.getTasks()
    }
}).mount();
