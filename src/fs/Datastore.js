"use strict";
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
var __importStar = (this && this.__importStar) || function (mod) {
    if (mod && mod.__esModule) return mod;
    var result = {};
    if (mod != null) for (var k in mod) if (Object.hasOwnProperty.call(mod, k)) result[k] = mod[k];
    result["default"] = mod;
    return result;
};
Object.defineProperty(exports, "__esModule", { value: true });
const fs_1 = __importDefault(require("fs"));
const outil = __importStar(require("@utils/obj"));
class Datastore {
    constructor(p, handle, options) {
        this.options = outil.mergeDefault(Datastore.defaultOptions, options);
        this.handle = handle;
        this.path = p;
        this.state = null;
        this.ready = false;
        this.synced = false;
    }
    static async create(p, options) {
        return await new Datastore(p, await fs_1.default.promises.open(p, 'w+'), options).init();
    }
    async init() {
        if (this.ready)
            throw new Error('Cannot initialize state more than once');
        this.state = await this.resolveState();
        this.ready = true;
        this.synced = true;
        return this;
    }
    get(p) {
        if (!this.ready)
            throw new Error('Unable to read/modify state');
        const out = outil.get(this.state, p);
        this.synced = false;
        return out;
    }
    set(p, value) {
        if (!this.ready)
            throw new Error('Unable to read/modify state');
        outil.set(this.state, p, value);
        this.synced = false;
    }
    has(p) {
        if (!this.ready)
            throw new Error('Unable to read/modify state');
        const out = outil.has(this.state, p);
        this.synced = false;
        return out;
    }
    async write(force = false) {
        if (!this.ready)
            throw new Error('Cannot write state to disk');
        if (this.synced && !force)
            return false;
        await this.handle.writeFile(this.stringify(this.state), { flag: 'r+' });
        this.synced = true;
        return true;
    }
    async close(write = false) {
        if (!this.ready)
            throw new Error('Unable to destroy datastore');
        if (write)
            await this.write(true);
        await this.handle.close();
        this.ready = false;
        this.synced = false;
        this.state = null;
    }
    async resolveState() {
        let data;
        const wipe = this.options.wipeIfCorrupt;
        try {
            data = await this.handle.readFile({ flag: 'r+' });
            if (wipe)
                data = parseJSON(data);
        }
        catch (_a) {
            data = this.stringify(this.options.defaultState);
            await this.handle.writeFile(data, { flag: 'w+' });
            if (wipe)
                data = parseJSON(data);
        }
        finally {
            return wipe ? data : parseJSON(data);
        }
    }
    stringify(value) {
        return JSON.stringify(value, null, this.options.compact ? 0 : 2);
    }
}
Datastore.defaultOptions = {
    defaultState: {},
    wipeIfCorrupt: true,
    compact: false
};
exports.default = Datastore;
function parseJSON(text) {
    return JSON.parse(text.toString());
}
