import Command from '@core/Command';
import Melody from '@core/Melody';

const config: Melody.Config = require('./config.json');

process.on('unhandledRejection', (reason) => { throw reason; });

Melody.create(config).then((melody) => {
  melody.on('command', onCommand.bind(melody));
});

function onCommand(this: Melody, command: Command.DataBasic) {
  
}
