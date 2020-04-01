"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
var obj_1 = require("./obj");
exports.mergeDefault = obj_1.mergeDefault;
function wait(ms) {
    return new Promise((resolve) => setTimeout(resolve, ms));
}
exports.wait = wait;
function onceEvent(emitter, event) {
    return new Promise((resolve) => emitter.once(event, (...args) => resolve(args)));
}
exports.onceEvent = onceEvent;
function* zip(iterable1, iterable2) {
    const iter1 = iterable1[Symbol.iterator]();
    const iter2 = iterable2[Symbol.iterator]();
    while (true) {
        const result1 = iter1.next();
        const result2 = iter2.next();
        if (result1.done || result2.done)
            break;
        yield [result1.value, result2.value];
    }
}
exports.zip = zip;
