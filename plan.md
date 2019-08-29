# Bot Plan

## Objectives

* Create ;ping and ;destroy - **Done**
* Create ;help - **Done**
* Create ;configure - **Done**
* Add message logging functionality - **Done**
* Create ;dump and ;flush - Almost there
* Add database filestate persistency
* Create ;blacklist
* Add analytics tracking
* Create ;uptime and ;memory
* Create ;whatis and ;info
* Create ;mute
* Add autoMod and antiSpam functionality

## Roadmap

**Beta**: The bot will be in Beta until message logging is finished, then it will move into Alpha. Expected time to completion: About 1 Week.

**Alpha**: The bot will be in Alpha until muting, autoMod, antiSpam, and all current objectives are finished. Expected time to completion: About 4-6 Weeks.

**The Future**: Once out of Alpha, I will move the bot towards more fun things like CleverBot and Connect Four.

## Commands

```
;help|halp|h [command]
;adminhelp ['plugins'|'config'|'logging']
;feedback <@mention|user id>

;configure|config|cfg [config option] [value]
;dump [server name|server id] ['latest'|log number]
;flush [server name|server id]
;mute <@mention|user id>

;whatis|wi <id>
;info|i <'role'|'guild'|'user'|'channel'> [@mention|#mention|id]
;uptime
;memory

;blacklist <@mention|user id>
```