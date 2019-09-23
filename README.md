# Melody

This is a simple Discord bot, made by me, Scotty#4263. My lazy ass is still in the process of writing it, so I would not recommend using it. I really only have this public so I can show the horrible code to other people.

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
