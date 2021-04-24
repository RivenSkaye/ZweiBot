""" Zwei, the Discord utility bot

This is the core module, it contains base functions used throughout the bot
and performs checks on the config for which modules to load. It also contains
a few functions that proxy commands to different optional modules, like using
JSON or SQLite for data storage and such.

Author: Riven Skaye
"""
# Type hints and function signatures
from typing import List, Dict, Union, Any
from collections.abc import Callable
# Uptime
from datetime import datetime
import json ### Yeet this ###
# Discord stuff
import discord
from discord.ext import commands
# The cogs for this bot
import cogs

def _prefix(bot, msg):
    return commands.when_mentioned_or(bot.prefixes[str(guild)].append(";"))(bot, msg) if msg.guild else commands.when_mentioned_or(";")(bot, msg)

class ZweiBot(commands.Bot, case_insensitive=True): # No need to have it handle shards since we're small
    import utils._datastores as _ds
    def __init__(self):
        try:
            self._config = _ds.JSONStore("data/config.json")
            assert len(self._config.get(table="config", key="token")) > 0, "No token was provided, please add it to the config."
        except AssertionError as ae:
            print(ae)
            print("For your convenience, an empty `default_config.json` and `default_zweiDB.sdb` have been provided.")
            print("It is recommended to copy these to the same names without the `default_` prefix for a clean start.")
            exit(1)
        except Exception as ex:
            print(ex)
            print("Something went wrong trying to load data/config.json. Was it created properly?")
            exit(1)
        try:
            self._db = _ds.SQLiteStore(self._config.get("db"))
        except Exception as ex:
            print(ex)
            print(f"{self._config.get('db')} could not be opened. Please confirm the file exists.")
            exit(1)
        intents = discord.Intents.default()
        intents.presences = True
        super().__init__(command_prefix=_prefix,
                       description="Hey, I'm Zwei! So I hear you need someone to do a job for you.",
                       case_insensitive=True, strip_after_prefix=True, heartbeat_timeout=180.0,
                       intents=intents)
        self.bot_id = None # will be set in on_ready
        for cog in dir(cogs):
            if not cog.startswith("__"):
                # Load all cogs and pass self for any functions the cog may need
                self.add_cog(cog(self))

    async def on_ready(self):
        self.bot_id = self.user.id
        await self.change_presence(activity=discord.Activity(name="Living a life of freedom!", type=discord.ActivityType.custom))
        self.starttime = datetime.utcnow()

    @commands.Bot.Command(name="uptime", brief="List uptime for the bot", hidden=True)
    @commands.is_owner()
    async def uptime(self, ctx):
        ctx.send(str(datetime.utcnow() - self.starttime))
