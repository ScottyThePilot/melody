'use strict';
import Melody from './core/Melody.js';
import config from './config.js';

process.on('unhandledRejection', (reason) => { throw reason; });

import cmdHelp from './commands/help.js';
import cmdPing from './commands/ping.js';
import cmdRestart from './commands/restart.js';
import cmdStop from './commands/stop.js';

const commands = [
  cmdHelp,
  cmdPing,
  cmdRestart,
  cmdStop
];

Melody.create(config, commands).then((melody) => {
  process.on('SIGHUP', () => {
    melody.destroy().then(() => process.exit());
  });

  melody.on('message', async (message) => {
    if (melody.mention.test(message.content)) {
      const match = message.content.match(melody.mention)[0];
      const msg = message.content.slice(match.length).trim();
      const response = await melody.clever.send(message.channel.id, msg).catch(() => null);
      
      if (response === null) await message.react('\u274e').catch();
      else await message.channel.send(response).catch(melody.catcher);
    }
  });

  melody.on('command', async (data) => {
    const { message, command } = data;
    const found = melody.commands.find((cmd) => cmd.is(command));

    if (!found) return;

    const level = melody.getUserLevel(data);
    const manager = message.guild ? melody.managers.get(message.guild.id) : null;
    const where = message.guild ? 'guild' : 'dm';

    const code = await found.attempt({
      melody,
      level,
      where,
      manager,
      ...data
    });

    switch (code) {
      case 'disabled': return await message.channel.send('That command is disabled.');
      case 'not_here': return await message.channel.send('You cannot use this command here.');
      case 'wrong_level': return await message.channel.send('You do not have permission to do that.');
    }
  });
});
