""" Zwei, the Discord utility bot

This is the core module, it contains base functions used throughout the bot
and performs checks on the config for which modules to load. It also contains
a few functions that proxy commands to different optional modules, like using
JSON or SQLite for data storage and such.

Authors:    Riven Skaye
            TheJoseph98
            bthen13
"""
# Type hints and function signatures
from collections.abc import Callable
from utils._datastores import DataStore
# Uptime
from datetime import datetime
# Discord stuff
import discord
from discord.ext import commands

class ZweiBot(commands.AutoShardedBot):
    def __init__(self, cmd_prefix: Callable, config: DataStore, database: DataStore):
        self._config = config
        try:
            self._token = self._config.get_sync("config", "token")["token"]
            assert len(self._token) > 0, "No token was provided, please add it to the config."
        except AssertionError as ae:
            print(ae)
            exit(1)
        self._db = database
        intents = discord.Intents.all() # Change this if you don't need some
        super().__init__(command_prefix=cmd_prefix,
                         case_insensitive=True, strip_after_prefix=True,
                         heartbeat_timeout=180.0, intents=intents)
        self.bot_id = None # will be set in on_ready
        cogs = self._config.get_sync("init", "cogs")["cogs"]
        for cog in cogs.keys():
            self.load_extension(f"cogs.{cog}")

    async def on_ready(self):
        self.bot_id = self.user.id
        await self.change_presence(activity=discord.Activity(name="Living a life of freedom!", type=discord.ActivityType.custom))
        self.starttime = datetime.utcnow()
