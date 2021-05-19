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
            self._owners = frozenset(self._config.get_sync("config", "owners")["owners"])
            print(self._owners)
            self.get_owners = False
            supkw = {}
            if len(self._owners) > 1:
                owner_ids = self._owners
                supkw["owner_ids"] = self._owners
            elif len(self._owners) == 1:
                owner_id = self._owners
                supkw["owner_id"] = self._owners
            else:
                self.get_owners = True
            assert len(self._token) > 0, "No token was provided, please add it to the config."
        except AssertionError as ae:
            print(ae)
            exit(1)
        self._db = database
        intents = discord.Intents.all() # Change this if you don't need some
        super().__init__(command_prefix=cmd_prefix,
                         case_insensitive=True, strip_after_prefix=True,
                         heartbeat_timeout=180.0, intents=intents, **supkw)
        self.bot_id = None # will be set in on_ready
        cogs = self._config.get_sync("init", "cogs")["cogs"]
        for cog in cogs.keys():
            self.load_extension(f"cogs.{cog}")
        self.emotes = {}
        self.emotes.update(self._config.get_sync("init", "emotes")["emotes"])

    async def on_ready(self):
        if self.get_owners:
            await self.application_info()
        try:
            await self._db._conn()
        except Exception as ex:
            print("Couldn't open `./data/zweiDB.sdb`. Please make sure that the file exists.")
            print("if it doesn't, change the name of `default_zweiDB.sdb` for a clean start.")
            print("====================")
            print(ex)
            exit(1)
        self.bot_id = self.user.id
        await self.change_presence(activity=discord.Game(name="with Kuro | ;help", afk=False))
        self.starttime = datetime.utcnow()

    async def close(self):
        cfg = await self._config.close()
        dbc = await self._db.close()
        if not cfg and dbc:
            print("Couldn't close all database connections, PANIC!")
            exit(1)
        await super().close()

    def get_name(self, uid: int, guild: discord.Guild=None):
        """ Utility function that returns whatever name fits the context.

        Returns the user's current nickname if one is set in this guild, or
        returns their username if they're not in this server or don't have
        a nickname set currently.
        """
        mdr = {"~": r"\~", "_": r"\_", "*": r"\*", "|": r"\|", "`": r"\`", ">": r"\>"}
        if guild:
            fetch = guild.get_member(uid)
            if fetch:
                name = fetch.display_name
        else:
            name = self.get_user(uid).display_name
        for rep in mdr:
            name = name.replace(rep, mdr[rep])
        return name
