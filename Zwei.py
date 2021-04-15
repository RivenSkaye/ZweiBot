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
# Discord stuff
import discord
from discord.ext import commands
# The cogs for this bot
import cogs
# A list of classes in cogs goes here, so we can load them as cogs
init_cogs = ()

def _prefix(bot, guild):
    return bot.prefixes[str(guild)] + [f"<@{bot.bot_id}>", f"<@!{bot.bot_id}"]

class ZweiBot(commands.Bot): # No need to have it handle shards since we're small
    import utils._datastores as _ds
    def __init__(self):
        try:
            with open("./data/config.json") as config:
                self._config = json.load(config)
            assert len(self._config["token"]) > 0, "No token was provided, please add it to the config."
        except AssertionError as ae:
            print(ae.message)
            exit(1)
        except:
            print("Something went wrong trying to load ./data/config.json. Was it created properly?")
            exit(1)
        intents = discord.Intents.default()
        intents.presences = True
        super().__init__(command_prefix=self._prefix,
                       description="Hey, I'm Zwei! What can I do for you? Who do we ban today?",
                       case_insensitive=True, strip_after_prefix=True, heartbeat_timeout=180.0,
                       intents=intents)
        self.bot_id = None # will be set in on_ready
        for cog in init_cogs:
            # Load all cogs and pass self for any functions the cog may need
            self.add_cog(cog(self))

    async def on_ready(self):
        self.bot_id = self.user.id
        await self.change_presence(activity=discord.Activity(name="Living a life of freedom!", type=discord.ActivityType.custom))
