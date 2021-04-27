""" Zwei, the Discord utility bot

This is the core module, it contains base functions used throughout the bot
and performs checks on the config for which modules to load. It also contains
a few functions that proxy commands to different optional modules, like using
JSON or SQLite for data storage and such.

Author: Riven Skaye
"""
# Type hints and function signatures
from typing import Union, Optional
from collections.abc import Callable
from utils._datastores import DataStore
# Uptime
from datetime import datetime
import json ### Yeet this ###
# Discord stuff
import discord
from discord.ext import commands
# The cogs for this bot
import cogs

class ZweiBot(commands.Bot, case_insensitive=True): # No need to have it handle shards since we're small
    def __init__(self, cmd_prefix: Callable, config: DataStore, database: DataStore):
        self._config = config
        try:
            assert len(self._config.get(table="config", key="token")) > 0, "No token was provided, please add it to the config."
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
        # This is needed for user-based purging of messages
        self._purgetarget = None

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
        await ctx.send(f"I've been working my hardest for {uptime} now!")

    def is_author(self, msg) -> bool:
        return msg.author == self._purgetarget

    @commands.Bot.Command(name="purge", aliases=["prune","massdelete","massdel"])
    @commands.has_permissions(manage_messages=True)
    async def purge(self, ctx, amount: int=5, user: Optional[Union[discord.abc.User,int]]=None):
        if amount < 1:
            await ctx.send("Could you stop trying to purge thin air?")
            return
        elif amount > 250:
            await ctx.send("Please keep the amount of messages to be purged somewhat manageable.\nNo more than 250 at a time, okay?")
            return
        remainder = amount
        user = await ctx.guild.get_member(user) if isinstance(user, int) else user
        # None == None will always return True, so no user means any author
        self._purgetarget = user
        while remainder > 0:
            limit = 100 if remainder > 99 else remainder
            await ctx.channel.purge(limit=limit, check=self._is_author, bulk=True, oldest_first=True)
            remainder = remainder - limit
        usermsg = f" sent by {user.name}" if user else ""
        await ctx.send(f"I deleted the last {amount} of messages{usermsg} for you." if amount > 1 else "I deleted the last message{usermsg}. You could've done that faster manually.")
