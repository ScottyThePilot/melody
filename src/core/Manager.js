"use strict";
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", { value: true });
const Datastore_1 = __importDefault(require("@fs/Datastore"));
const Logger_1 = __importDefault(require("@fs/Logger"));
const Queue_1 = __importDefault(require("@utils/Queue"));
const fs_1 = __importDefault(require("fs"));
const path_1 = __importDefault(require("path"));
class Manager {
    constructor(id, logger, store) {
        this.id = id;
        this.logger = logger;
        this.store = store;
        this.queue = new Queue_1.default();
    }
    static async create(id, location, defaultState) {
        const folder = path_1.default.join(location.toString(), id);
        await fs_1.default.promises.mkdir(folder);
        const logger = await Logger_1.default.create(path_1.default.join(folder, 'latest.log'), {
            logsFolder: path_1.default.join(location.toString(), id, 'logs')
        });
        const store = await Datastore_1.default.create(path_1.default.join(folder, 'store.json'), {
            defaultState
        });
        return new Manager(id, logger, store);
    }
    log(header, text, ...rest) {
        return this.logger.log(header, text, ...rest);
    }
    get(p) {
        return this.store.get(p);
    }
    set(p, value) {
        this.store.set(p, value);
    }
    has(p) {
        return this.store.has(p);
    }
    write(force = false) {
        return this.queue.wait(() => this.store.write(force));
    }
    async destroy(write = false) {
        await Promise.all([
            this.logger.close(),
            this.store.close(write)
        ]);
    }
}
exports.default = Manager;
