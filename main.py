#!/usr/bin/env python3
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

__version__ = '0.1'

from . import Zwei
from .utils import _datastores as _ds
from discord.ext import commands

try:
    # If you're not using JSON, good luck figuring it out!
    bot_conf = _ds.JSONStore("data/config.json")
    bot_db = _ds.SQLiteStore("./data/zweiDB.sdb")
except Exception as ex:
    print("For your convenience, an empty `default_config.json` and `default_zweiDB.sdb` have been provided.")
    print("It is recommended to copy these to the same names without the `default_` prefix for a clean start.")
    print("====================")
    print(ex)
    exit(1)

def get_prefix(bot, msg):
    prefix = ";"
    if msg.guild: # If it was sent from a guild
        key = str(msg.guild.id)
        guilds = bot._config.get_sync(table="prefixes", key=key)
        if key in guilds.keys():
            prefix = guilds[key]
    return commands.when_mentioned_or(prefix)(bot,msg)

zwei = Zwei.ZweiBot(cmd_prefix=get_prefix, config=bot_conf, database=bot_db)
try:
    # Run the bot. This blocks every line of code below it until it closes
    zwei.run(zwei._token)
# There's nothing that should go wrong here. I think.
except Exception as wtf:
    print("Zwei encountered a MAJOR malfunction, contact the devs for help!")
    print("====================")
    print(wtf)
    exit(1)

# If we reached all the way here, exit normally without error status
print("Thanks for using Zwei and don't forget to update!")
exit(0)
