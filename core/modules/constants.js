'use strict';

const MUTE_RESPONSES = [
  'I\'m afraid I can\'t let you do that. Send messages slower next time.',
  'Looks like you\'re sending messages a little too quickly.',
  'Please slow down, you\'re sending messages awful quickly.',
  'Please calm down, you\'re upsetting the robo-hampsters.',
  'Looks like you were sending messages too quickly.',
  'Try not to send messages so fast next time :(',
  'Whoa there! Slow down with the messages.',
  'Please don\'t send messages so quickly :(',
  'Next time, send messages a bit slower.',
  'Oh! So that\'s what that button does...',
  'It\'s rude to send messages so quickly.',
  'Enhance your calm.'
];

const MUTE_NOTICE = 'You were automatically muted for spamming. If you believe this is a bug, please contact this bot\'s owner, Scotty#4263';

const CFG_INVALID_SUBCOMMAND = `Please specify a valid subcommand. Valid subcommands are: \`list\`, \`get\` and \`set\`. Use \`{0}configure <subcommand>\` for more information about a subcommand.`;

const CFG_LOGS_NOTICE = `Logs can be retrieved with \`{0}dump\``;
const CFG_MUTE_NOTICE = `In order for automated muting to work, a \`mutedRole\` must be specified.`;

const CFG_PROP_DESCRIPTIONS = {
  logMessages: `If \`logMessages\` is true, the bot will log all sent messages. ${CFG_LOGS_NOTICE}`,
  logMessageChanges:  `If \`logMessageChanges\` is true, the bot will log message edits and deletions. ${CFG_LOGS_NOTICE}`,
  mutedRole: `If \`mutedRole\` is specified, the \`{0}mute\` command can be used to mute users. ${CFG_MUTE_NOTICE}`
};

const BOOLEAN_KEYWORDS = {
  'true': true,
  't': true,
  'false': false,
  'f': false,
  'enable': true,
  'enabled': true,
  'e': true,
  'disable': false,
  'disabled': false,
  'd': false,
  'on': true,
  'off': false,
  'yes': true,
  'y': true,
  'no': false,
  'n': false,
  '0': false,
  '1': true
};

module.exports = {
  MUTE_RESPONSES,
  MUTE_NOTICE,
  BOOLEAN_KEYWORDS,
  CFG_INVALID_SUBCOMMAND,
  CFG_LOGS_NOTICE,
  CFG_MUTE_NOTICE,
  CFG_PROP_DESCRIPTIONS
};
