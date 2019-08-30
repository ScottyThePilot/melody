# Melody

This is a simple Discord bot, made by me, Scotty#4263. My lazy ass is still in the process of writing it, so I would not recommend using it. I really only have this public so I can show the code to other people.

## The Plan

### Objectives

* Create ;ping and ;destroy - **Done**
* Create ;help - **Done**
* Create ;configure - **Done**
* Add message logging functionality - **Done**
* Create ;dump - **Done**
* Create ;flush
* Add database filestate persistency - **Done**
* Create ;blacklist
* Add analytics tracking
* Create ;uptime and ;memory
* Create ;whatis and ;info
* Create ;mute
* Add autoMod and antiSpam functionality

### Completed Side Objectives

* Added a parent process to monitor the bot, log any errors and restart on zero exit codes.

### Roadmap

**Beta**: Logging has been (Mostly) Finished

**Alpha**: The bot will be in Alpha until muting, autoMod, antiSpam, and all current objectives are finished. Expected time to completion: About 4-6 Weeks.

**The Future**: Once out of Alpha, I will move the bot towards more fun things like CleverBot and Connect Four.

### Current & Planned Commands

Core
```
;ping
;help|helpall|halp|h [command]
;adminhelp ['plugins'|'config'|'logging']
;feedback <feedback>
;configure|config|cfg [config option] [value]
;dump [server name|server id] ['latest'|log number]
;flush [server name|server id]
```

Moderation
```
;mute <@mention|user id>
```

Utility
```
;whatis|wi <id>
;info|i <'role'|'guild'|'user'|'channel'> [@mention|#mention|id]
;uptime
;memory
```

Owner
```
;blacklist <@mention|user id>
;stop
;restart
```