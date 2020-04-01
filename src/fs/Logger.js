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
const path_1 = __importDefault(require("path"));
const tutil = __importStar(require("@utils/text"));
const util = __importStar(require("@utils/util"));
class Logger {
    constructor(p, stream, options) {
        this.options = util.mergeDefault(Logger.defaultOptions, options);
        this.stream = stream;
        this.path = p;
        this.ready = false;
    }
    static async create(p, options) {
        const stream = fs_1.default.createWriteStream(p, { flags: 'a' });
        await util.onceEvent(stream, 'ready');
        return await new Logger(p, stream, options).init();
    }
    async init() {
        if (this.ready)
            throw new Error('Cannot initialize state more than once');
        if (this.options.logsFolder !== null) {
            await fs_1.default.promises.mkdir(this.options.logsFolder, { recursive: true });
            await this.rotate();
        }
        this.ready = true;
        return this;
    }
    async rotate() {
        if (this.options.logsFolder === null)
            return false;
        const now = new Date();
        this.stream.cork();
        const handle = await fs_1.default.promises.open(this.path, 'r+');
        const { size } = await handle.stat();
        const rotate = size >= this.options.maxFileSize;
        if (rotate) {
            const folder = this.options.logsFolder.toString();
            const filepath = path_1.default.join(folder, tutil.savifyDate(now) + '.log');
            const contents = await handle.readFile();
            await fs_1.default.promises.writeFile(filepath, contents, { flag: 'wx' });
            await handle.writeFile('');
        }
        await handle.close();
        this.stream.uncork();
        return rotate;
    }
    log(header, text, ...rest) {
        const entry = tutil.makeLogEntry(header, text, ...rest);
        if (this.options.logToConsole)
            console.log(entry);
        return this.stream.writable ? this.stream.write(entry + '\n') : true;
    }
    async close() {
        this.stream.end();
        this.ready = false;
        await util.onceEvent(this.stream, 'finish');
    }
}
Logger.defaultOptions = {
    logToConsole: false,
    logsFolder: null,
    maxFileSize: 524288
};
exports.default = Logger;
