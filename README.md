# Melody

This is a simple Discord bot, made by me, Scotty#4263. My lazy ass is still in the process of writing it, so I would not recommend using it. I really only have this public so I can show the horrible code to other people.

If you want to run your own copy of my bot to learn how it works, to provide feedback, to investigate issues, or to simply check out my bot, feel free to clone or fork my repo, and follow the instructions below on how to host it. 

## The Plan

### Current Objective: AutoMod

I'm currently in the process of adding an "auto-moderator" to my bot to allow server owners to filter spam and automate other moderation-related tasks. The main thing I'm focusing on adding to AutoMod is a spam filter. Just like nearly everything server-related on my bot, this will be toggleable and configurable.

Completion goal: 9/28/2019

### Previous Objective: Scheduled Tasks

For a while, I had very much been in the need of a way to scheduled tasks. I knew this almost since the beginning, since `node-schedule` sat unused in my dependencies for well over a month or two. You won't notice it, but behind the scenes, melody is running scheduled tasks such as:
* Daily reports on memory usage and Discord API ping
* Randomly cycled activity messages
* Bi-hourly checks on logs to make sure they're not exceeding their maximum file size

Completed on: 9/18/2019

## How does it work?

`index.js` serves to keep the bot alive by forking off a child node process (which is the bot), restarting it when it exits with 0, and exiting when the child process exits with 1. This version of Melody works around a `Bot` class. `Bot` instances contain a `Discord.Client`, a `Logger`, a `GuildManager` for each guild, and `Command`s for each command. This instance bundles everything important into one object so only a single variable has to be passed around for full access to the bot. Upon receiving the `'ready'` event from the Discord API, the bot initializes its data directories, loads each `GuildManager` instance, builds all of the commands from the commands directory, and sets up scheduled tasks. From there it listens to input and reacts accordingly.

## File Structure and Config

This diagram only shows the important files needed for the bot to work. Some files and folders are omitted to reduce size.
The data folder does not have to be included, as it is generated automatically by the bot if it doesn't already exist.
```
melody_v3/
├─core/
│ ├─commands/
│ │ └─[...]
│ ├─data/                 [Not included]
│ │ ├─guilds/
│ │ │ └─[...]
│ │ ├─blacklist.json
│ │ ├─logs/
│ │ └─main.log
│ ├─modules/
│ │ └─[...]
│ ├─subfunctions/
│ │ └─[...]
│ ├─botEvents.js
│ ├─changeloglatest.json
│ ├─config.json           [Not included]
│ ├─melody.js
│ └─setup.js
├─index.js
└─start.bat
```

Here's an example of what config.js might look like
```json
{
  "version": ["Stable", "1.0.0"],
  "prefix": "!",
  "token": "your-token-goes-here",
  "trustedUsers": [
    "a-discord-id-of-someone-you-trust"
  ]
}
```