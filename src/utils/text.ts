import Discord from 'discord.js';
import moment from 'moment';
import { inspect } from 'util';

export function logifyDate(date?: Date): string {
  return '[' + moment(date).format('YYYY-MM-DD HH:mm:ss.SSS [UTC]ZZ') + ']';
}

export function savifyDate(date?: Date): string {
  return logifyDate(date).slice(1, 24).replace(/[^0-9]+/g, '_'); 
}

export function makeLogEntry(header: string, text: string = '', ...rest: string[]): string {
  const h = header ? `[${('' + header).toUpperCase()}] ` : '';
  const r = rest.length ? ':\n' + rest.map(e => '  ' + e).join('\n') : '';
  const data = text.trim() + r.trim();
  return logifyDate() + (h.length || data.length ? ': ' : '') + h + data;
}

export function logifyUser(entity: Discord.GuildMember | Discord.User): string {
  const user = entity instanceof Discord.GuildMember ? entity.user : entity;
  return `${user.tag} (${user.id})` + (user.bot ? ' (BOT)' : '');
}

export function logifyGuild(guild: Discord.Guild): string {
  return logify(guild) + (guild.available ? '' : ' (Unavailable)');
}

export function logifyError<E extends Error>(err: E): string {
  const info = err instanceof Discord.HTTPError || err instanceof Discord.DiscordAPIError ? `(${err.code} ${err.path})` : '';
  const message = joinAny(err.message, info) || inspect({ ...err }, true);
  return `${err.name || 'Unknown Error'}: ${message}`;
}

export function logify({ name, id }: { name: string, id: string }): string {
  return `${name} (${id})`;
}

export function capFirst(str: string): string {
  if (str.length === 0) return '';
  const a = String.fromCodePoint(str.codePointAt(0) as number).toUpperCase();
  return a + str.slice(a.length).toLowerCase();
}

export function joinAny(...strings: any[]): string {
  let out = '';
  for (const str of strings) {
    if (typeof str !== 'string' || !str.trim()) continue;
    if (out) out += ' ';
    out += str.trim();
  }
  return out;
}
