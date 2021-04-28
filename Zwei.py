""" Zwei, the Discord utility bot

This is the core module, it contains base functions used throughout the bot
and performs checks on the config for which modules to load. It also contains
a few functions that proxy commands to different optional modules, like using
JSON or SQLite for data storage and such.

Authors:    Riven Skaye
            TheJoseph98
            bthen13
"""
import asyncio
# Type hints and function signatures
from typing import Union, Optional
from collections.abc import Callable
from utils._datastores import DataStore
# Uptime
from datetime import datetime
# Discord stuff
import discord
from discord.ext import commands
# The cogs for this bot
import cogs

class ZweiBot(commands.Bot, case_insensitive=True): # No need to have it handle shards since we're small
    def __init__(self, cmd_prefix: Callable, config: DataStore, database: DataStore):
        self._config = config
        try:
            self._token = asyncio.run(self._config.get("config", "token"))
            assert self._token > 0, "No token was provided, please add it to the config."
        except AssertionError as ae:
            print(ae)
            exit(1)
        self._db = database
        intents = discord.Intents.all() # Change this if you don't need some
        super().__init__(command_prefix=cmd_prefix,
                         case_insensitive=True, strip_after_prefix=True,
                         heartbeat_timeout=180.0, intents=intents)
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
        """Lists total uptime for the bot, in a human readable format.

        Lists the time as `xx days, hh:mm:ss`. This can be used indicatively
        for debugging purposes (for example long term usage filling up memory)
        and as a gimmick for the owners. Honestly, it's just here to test the
        decorators and to check if the bot functions at all.
        """
        uptime = str(datetime.utcnow() - self.starttime)
        await ctx.send(f"I've been giving it my all for {uptime} now!")

    @commands.Bot.Command(name="prefix")
    @commands.guild_only()
    async def prefix(self, ctx, *, new_prefix: Optional[str]=None):
        """ Tells you the bot's current prefix in this server, or changes it.

        If the command is used without any arguments, Zwei will reply with the
        current prefix. If any arguments are given, the prefix will be changed
        to whatever the user sent.
        Before processing, any sequence of `set ` will be removed. The keyword
        is NOT required for changing the prefix, but a lot of users are idiots.
        """
        key = str(ctx.guild.id)
        server_prefix = await self._conf.get(table="prefixes", key=key)
        pref = ";" if server_prefix["error"] else server_prefix[key]
        pref.replace("`", "\`") # Display failsafe
        if not new_prefix:
            await ctx.reply(f"I'll be listening to any messages starting with `{pref}`.")
            return
        else:
            has_perm = await ctx.message.author.guild_permissions.manage_guild
            if not has_perm:
                await ctx.reply(f"Changing my prefix requires the `Manage Server` permission.\nYou don't seem to have this, so I'll keep listening to `{pref}` for now.")
            new_prefix.replace("set ", "") # A load of people do this, smh
            if server_prefix["error"]:
                success = await self._config.set(table="prefixes", data=new_prefix, key=key)
            else:
                success = await self._config.update(table="prefixes", data=new_prefix, key=key)

            if success:
                new_prefix = new_prefix.replace("`", "\`")
                await ctx.send(f"Prefix changed to `{new_prefix}`")
            else:
                await ctx.send("I couldn't change the prefix, something went wrong!\nContact the devs for assistance")
