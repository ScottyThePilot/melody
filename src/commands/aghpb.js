'use strict';
import Command from '../core/Command.js';
import config from '../config.js';
import aghpb from 'aghpb_get';

let list = null;

export default new Command({
  name: 'aghpb',
  help: {
    short: 'Gets an image of an anime girl holding a programming book.',
    long: 'Gets a random image of an anime girl holding a programming book sourced from '
      + 'https://github.com/laynH/Anime-Girls-Holding-Programming-Books.',
    usage: `${config.prefix}aghpb`,
    example: `${config.prefix}aghpb`
  },
  aliases: ['animegirlholdingprogrammingbook'],
  exec: async function exec({ melody, message }) {
    await message.channel.send(await pickImage()).catch(melody.catcher);
  }
});

async function pickImage() {
  if (list === null) list = await aghpb();
  return list[Math.floor(Math.random() * list.length)];
}
