"use strict";
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", { value: true });
const discord_js_1 = __importDefault(require("discord.js"));
const moment_1 = __importDefault(require("moment"));
const util_1 = require("util");
function logifyDate(date) {
    return '[' + moment_1.default(date).format('YYYY-MM-DD HH:mm:ss.SSS [UTC]ZZ') + ']';
}
exports.logifyDate = logifyDate;
function savifyDate(date) {
    return logifyDate(date).slice(1, 24).replace(/[^0-9]+/g, '_');
}
exports.savifyDate = savifyDate;
function makeLogEntry(header, text = '', ...rest) {
    const h = header ? `[${('' + header).toUpperCase()}] ` : '';
    const r = rest.length ? ':\n' + rest.map(e => '  ' + e).join('\n') : '';
    const data = text.trim() + r.trim();
    return logifyDate() + (h.length || data.length ? ': ' : '') + h + data;
}
exports.makeLogEntry = makeLogEntry;
function logifyUser(entity) {
    const user = entity instanceof discord_js_1.default.GuildMember ? entity.user : entity;
    return `${user.tag} (${user.id})` + (user.bot ? ' (BOT)' : '');
}
exports.logifyUser = logifyUser;
function logifyGuild(guild) {
    return logify(guild) + (guild.available ? '' : ' (Unavailable)');
}
exports.logifyGuild = logifyGuild;
function logifyError(err) {
    const info = err instanceof discord_js_1.default.HTTPError || err instanceof discord_js_1.default.DiscordAPIError ? `(${err.code} ${err.path})` : '';
    const message = joinAny(err.message, info) || util_1.inspect({ ...err }, true);
    return `${err.name || 'Unknown Error'}: ${message}`;
}
exports.logifyError = logifyError;
function logify({ name, id }) {
    return `${name} (${id})`;
}
exports.logify = logify;
function capFirst(str) {
    if (str.length === 0)
        return '';
    const a = String.fromCodePoint(str.codePointAt(0)).toUpperCase();
    return a + str.slice(a.length).toLowerCase();
}
exports.capFirst = capFirst;
function joinAny(...strings) {
    let out = '';
    for (const str of strings) {
        if (typeof str !== 'string' || !str.trim())
            continue;
        if (out)
            out += ' ';
        out += str.trim();
    }
    return out;
}
exports.joinAny = joinAny;
