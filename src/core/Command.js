"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
class Command {
    constructor({ name, help, run, aliases = [], info = {}, where = 'anywhere', disabled = false, level = 0, hidden = false }) {
        this.name = name;
        this.help = help;
        this.aliases = aliases;
        this.info = info;
        this.where = where;
        this.disabled = disabled;
        this.level = level;
        this.hidden = hidden;
        this.run = run;
    }
    get group() {
        if (typeof this.level === 'string')
            return this.level;
        switch (this.level) {
            case 0: return 'Everyone';
            case 1: return 'Server Administrators';
            case 2: return 'Server Owners';
            case 3: return 'Trusted Users';
            case 10: return 'Bot owner';
            default: return 'Unknown';
        }
    }
    async attempt(data) {
        if (this.disabled)
            return false;
        if (!this.here(data.location))
            return false;
        if (data.level < this.level)
            return false;
        await this.run.call(this, data);
        return true;
    }
    here(location) {
        return this.where === 'anywhere' ? true : this.where === location;
    }
    is(query) {
        const q = query.toLowerCase();
        if (this.name === q)
            return true;
        for (const alias of this.aliases)
            if (alias.toLowerCase() === q)
                return true;
        return false;
    }
}
exports.default = Command;
