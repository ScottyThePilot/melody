'use strict';
import Command from '../core/Command.js';
import config from '../config.js';
import { inspect } from 'util';

export default new Command({
  name: 'eval',
  level: 10,
  help: {
    short: 'Run custom code remotely.',
    long: 'Instructs Melody to run the given code.',
    usage: `${config.prefix}eval <code>`,
    example: `${config.prefix}eval \`\`\`js\nreturn "hello world";\n\`\`\``
  },
  exec: async function exec(data) {
    let { melody, message, argsText } = data;
    let command = argsText.match(/^```js\s*([^]+)\s*```$/);
    if (command === null) {
      await message.channel.send('Could not parse code block').catch(melody.catcher);
    } else {
      command = command[1].trim();
      if (command.length === 0) {
        await message.channel.send('Nothing was provided').catch(melody.catcher);
      } else {
        console.log(`Running Eval: \`${command}\``);
        let msg = await doEval(command, data);
        while (msg.includes(melody.token))
          msg = msg.replace(melody.token, '[Token');
        await message.channel.send(msg).catch(melody.catcher);
      }
    }
  }
});

const MSG_SIZE_LIMIT = 2000 - 20;

async function doEval(context, data) {
  let message;
  try {
    const result = await customEval(context, data);
    message = inspect(result);
    if (message.length > MSG_SIZE_LIMIT)
      message = message.slice(0, MSG_SIZE_LIMIT) + '\n...';
    message = `Result:\n\`\`\`${message}\`\`\``;
  } catch (err) {
    message = inspect(err);
    if (message.length > MSG_SIZE_LIMIT)
      message = message.slice(0, MSG_SIZE_LIMIT) + '\n...';
    message = `Error:\n\`\`\`${message}\`\`\``;
  }

  return message;
}

function customEval(context, data) {
  return eval(context); // jshint ignore: line
}
