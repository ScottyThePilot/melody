"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
class Group extends Set {
    /**
     * Returns an array containing the same elements as this Group.
     */
    array() {
        return [...this.values()];
    }
    /**
     * Returns true if every element passes the check specified by
     * the `fn` parameter, and false otherwise.
     */
    every(fn) {
        for (let e of this)
            if (!fn(e, this))
                return false;
        return true;
    }
    /**
     * Returns true if at least one element passes the check specified
     * by the `fn` parameter, and false otherwise.
     */
    some(fn) {
        for (let e of this)
            if (fn(e, this))
                return true;
        return false;
    }
    /**
     * Returns a new Group containing only elements that passed the
     * check specified by the `fn` parameter.
     */
    filter(fn) {
        let out = new Group();
        for (let e of this)
            if (fn(e, this))
                out.add(e);
        return out;
    }
    /**
     * Returns the first element in the Group that passes the check
     * specified by the `fn` parameter. If no elements pass the check,
     * undefined is returned.
     */
    find(fn) {
        for (let e of this)
            if (fn(e, this))
                return e;
        return;
    }
    /**
     * Creates a new Group with the results of calling the provided
     * function on every element.
     */
    map(fn) {
        let out = new Group();
        for (let e of this)
            out.add(fn(e, this));
        return out;
    }
}
exports.default = Group;
