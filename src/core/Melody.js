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
const Command_1 = __importDefault(require("@core/Command"));
const Manager_1 = __importDefault(require("@core/Manager"));
const Logger_1 = __importDefault(require("@fs/Logger"));
const Group_1 = __importDefault(require("@utils/Group"));
const Table_1 = __importDefault(require("@utils/Table"));
const tutil = __importStar(require("@utils/text"));
const util = __importStar(require("@utils/util"));
const Discord = __importStar(require("discord.js"));
const events_1 = require("events");
const fs_1 = __importDefault(require("fs"));
const path_1 = __importDefault(require("path"));
const events = Object.values(Discord.Constants.Events);
class Melody extends events_1.EventEmitter {
    constructor(client, options) {
        super();
        const { logger, commands, managers, config: { version, token, prefix, owner, trustedUsers, paths } } = options;
        this.ready = false;
        this.client = client;
        this.logger = logger;
        this.commands = commands;
        this.managers = managers;
        this.version = version;
        this.token = token;
        this.prefix = prefix;
        this.owner = owner;
        this.trustedUsers = trustedUsers;
        this.paths = paths;
    }
    static async create(config) {
        const client = new Discord.Client(config.client);
        for (const p of ['data', 'commands'])
            await fs_1.default.promises.mkdir(config.paths[p], { recursive: true });
        await fs_1.default.promises.mkdir(path_1.default.join(config.paths.data, 'guilds'));
        const logger = await Logger_1.default.create(path_1.default.join(config.paths.data, 'main.log'), {
            logsFolder: path_1.default.join(config.paths.data, 'logs'),
            logToConsole: true
        });
        const managers = new Table_1.default();
        const commands = new Group_1.default();
        return await new Melody(client, { logger, managers, commands, config }).init();
    }
    async init() {
        var _a;
        await Promise.all([
            util.onceEvent(this.client, 'ready'),
            this.client.login(this.token)
        ]);
        this.log('INFO', 'Connection established');
        await util.wait(1000);
        for (const event of events) {
            if (event === 'message')
                continue;
            this.client.on(event, (...args) => {
                this.emit(event, ...args);
            });
        }
        this.client.on('message', (message) => {
            const parsed = this.parseCommand(message);
            if (parsed)
                this.emit('command', parsed);
            else
                this.emit('message', message);
        });
        for (const guild of this.client.guilds.cache.values()) {
            await this.loadManager(guild.id);
            this.logger.log('DATA', `Guild ${tutil.logifyGuild(guild)} loaded`);
        }
        for (const file of await fs_1.default.promises.readdir(this.paths.commands)) {
            const location = path_1.default.join(this.paths.commands, file.toString());
            this.loadCommandAt(location);
        }
        this.logger.log('DATA', `${this.commands.size} Commands loaded`);
        await ((_a = this.client.user) === null || _a === void 0 ? void 0 : _a.setActivity('waiting...'));
        this.logger.log('INFO', 'Bot ready!');
        this.ready = true;
        return this;
    }
    get mention() {
        return new RegExp(`^<@!?${this.client.user.id}>\\s*`);
    }
    log(header, text, ...rest) {
        return this.logger.log(header, text, ...rest);
    }
    catcher(error) {
        const text = error instanceof Error ? tutil.logifyError(error) : error;
        this.logger.log('WARN', 'Caught an error', text);
    }
    async loadManager(id) {
        const folder = path_1.default.join(this.paths.data, 'guild');
        const manager = await Manager_1.default.create(id, folder);
        this.managers.set(id, manager);
    }
    async unloadManager(id) {
        const manager = this.managers.get(id);
        if (!manager)
            throw new Error('Cannot find manager with id ' + id);
        await manager.destroy();
    }
    loadCommandAt(location) {
        const command = requireRoot(location);
        if (command instanceof Command_1.default)
            this.commands.add(command);
    }
    parseCommand(message, prefixOverride) {
        if (message.author.bot)
            return null;
        const content = message.content.trim();
        const prefix = prefixOverride || this.prefix;
        if (!content.startsWith(prefix))
            return null;
        // Dissallow whitespace between the prefix and command name
        if (/^\s+/.test(content.slice(prefix.length)))
            return null;
        let args = content.slice(prefix.length).trim().split(/\s+/g);
        if (args.length < 1)
            return null;
        const command = args.shift().toLowerCase();
        const argsText = content.slice(prefix.length + command.length).trim();
        return { message, command, args, argsText };
    }
    async destroy() {
        await Promise.all([
            this.logger.close(),
            this.managers.valuesGroup().map((m) => m.destroy())
        ]);
    }
}
exports.default = Melody;
function requireRoot(id) {
    return require(path_1.default.join(process.cwd(), id));
}
