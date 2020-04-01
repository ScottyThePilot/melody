"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
const events_1 = require("events");
class Queue extends events_1.EventEmitter {
    constructor() {
        super();
        this.items = [];
        this.on('pop', () => {
            if (this.items.length)
                this.go();
        });
        this.on('add', () => {
            if (this.items.length === 1)
                this.go();
        });
    }
    get size() {
        return this.items.length;
    }
    get next() {
        return this.items[0];
    }
    add(item) {
        this.items.push(item);
        this.emit('add');
    }
    wait(item) {
        return new Promise((resolve, reject) => {
            this.add(() => item().then(resolve).catch(reject));
        });
    }
    async go() {
        const item = this.next;
        try {
            this.emit('start');
            await item();
            this.items.shift();
            this.emit('pop');
        }
        catch (error) {
            this.emit('error', error);
        }
    }
}
exports.default = Queue;
