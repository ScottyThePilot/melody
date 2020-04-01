"use strict";
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", { value: true });
const Group_1 = __importDefault(require("./Group"));
class Table extends Map {
    /**
     * Returns true if every entry passes the check specified by
     * the `fn` parameter, and false otherwise.
     */
    every(fn) {
        for (const [k, v] of this)
            if (!fn(v, k, this))
                return false;
        return true;
    }
    /**
     * Returns true if at least one entry passes the check specified
     * by the `fn` parameter, and false otherwise.
     */
    some(fn) {
        for (const [k, v] of this)
            if (fn(v, k, this))
                return true;
        return false;
    }
    /**
     * Returns a new Table containing only elements that passed the
     * check specified by the `fn` parameter.
     */
    filter(fn) {
        let out = new Table();
        for (let [k, v] of this)
            if (fn(v, k, this))
                out.set(k, v);
        return out;
    }
    /**
     * Returns the first entry in the Table that passes the check
     * specified by the `fn` parameter. If no entries pass the check,
     * undefined is returned.
     */
    find(fn) {
        for (const [k, v] of this)
            if (fn(v, k, this))
                return [k, v];
        return;
    }
    /**
     * Creates a new Table with the results of calling the provided
     * function on every value.
     */
    map(fn) {
        let out = new Table();
        for (const [k, v] of this)
            out.set(k, fn(v, k, this));
        return out;
    }
    /**
     * Creates a new Group with all of this Table's values.
     */
    valuesGroup() {
        return new Group_1.default(this.values());
    }
    /**
     * Creates a new Group will all of this Table's keys.
     */
    keysGroup() {
        return new Group_1.default(this.keys());
    }
}
exports.default = Table;
