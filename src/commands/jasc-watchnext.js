'use strict';
import Command from '../core/Command.js';
import config from '../config.js';

export default new Command({
  name: 'jasc:watchnext',
  help: {
    short: 'Gets a list of shows that the JASC server should watch next.',
    long: 'Gets a list of shows ranked based on how many members of the JASC server have marked '
      + 'them as plan to watch.',
    usage: `${config.prefix}jasc:watchnext`,
    example: `${config.prefix}jasc:watchnext`
  },
  where: 'guild',
  exclusive: '773040841082142751',
  exec: async function exec({ melody, message }) {
    await message.channel.send('sorry scotty command not finished yet').catch(melody.catcher);
  }
});
