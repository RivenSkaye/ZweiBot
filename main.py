""" This is the startup wrapper for Zwei. Run it as usual with Python stuff.

This wrapper mainly exists for those who wish to modify the bot for their own
needs, or for when expansion makes it a necessity for the bot to have certain
things initialized before she starts up. For example if logging further than
printing to `std.out` is ever added, this is the place to set all logging info
rather than cluttering up Zwei's code with environment stuff.

Authors:    RivenSkaye
            TheJoseph98
            bthen13
"""

import Zwei.Zweibot as ZB
import utils._datastores as _ds
from discord.ext import commands

try:
    bot_conf = _ds.JSONStore("data/config.json")
except Exception as ex:
    print("For your convenience, an empty `default_config.json` and `default_zweiDB.sdb` have been provided.")
    print("It is recommended to copy these to the same names without the `default_` prefix for a clean start.")
    print("====================")
    print(ex)
    exit(1)
try:
    bot_db = _ds.SQLiteStore(self._config.get("data/zweiDB.sdb"))
except Exception as ex:
    print("Couldn't open `./data/zweiDB.sdb`. Please make sure that the file exists.")
    print("if it doesn't, change the name of `default_zweiDB.sdb` for a clean start.")
    print("====================")
    print(ex)
    exit(1)

def get_prefix(bot, msg):
    prefix = ";"
    if msg.guild: # If it was sent from a guild
        key = str(msg.guild.id)
        guilds = bot_conf.get(table="prefixes", key=key)
        if key in guilds.keys():
            prefix = guilds[key]
    return commands.when_mentioned_or(prefixes)(bot,msg)

zwei = ZB(cmd_prefix=get_prefix, config=bot_conf, database=bot_db)
try:
    zwei.run(bot_conf.get("token"))
except Exception as wtf:
    print("Zwei encountered a MAJOR malfunction, contact the devs for help!")
    print("====================")
    print(wtf)
