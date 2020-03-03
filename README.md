# Melody

This is a simple Discord bot, made by me, Scotty#4263. My lazy ass is still in the process of writing it, so I would not recommend using it. I really only have this public so I can show the horrible code to other people.

If you want to run your own copy of my bot to learn how it works, to provide feedback, to investigate issues, or to simply check out my bot, feel free to clone or fork my repo, and follow the instructions below on how to host it. 

## File Structure and Config

This diagram only shows the important files needed for the bot to work. Some files and folders are omitted to reduce size.
The data folder does not have to be included, as it is generated automatically by the bot if it doesn't already exist.
```
melody_v3/
├─src/
│ ├─commands/
│ ├─core/
│ ├─config.json/
│ └─melody.js
├─index.js
└─start.bat
```

Here's an example of what config.js might look like
```json
{
  "version": "1.0.0",
  "prefix": "!",
  "token": "your-token-goes-here",
  "owner": "your-discord-id-here",
  "trustedUsers": [
    "a-discord-id-of-someone-you-trust"
  ]
}
```
