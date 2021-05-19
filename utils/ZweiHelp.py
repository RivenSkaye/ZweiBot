from discord.ext import commands
import discord
from typing import Optional

class ZweiHelp(commands.HelpCommand, name="Help"):
    """ The help command for Zwei and the functions used to build it up.

    This uses functions that shouldn't be in the normal bot code and it should
    not be imported as a cog with the other classes. Instead this gets loaded
    statically from main.py to ensure the bot has this cog.
    Like other cogs, this thing will be exposed to the reloading mechanisms to
    ensure the ability to update it without rebooting the bot.
    """
    def __init__(self, bot):
        self.bot = bot

    @commands.command(name="help", hidden=True)
    async def help(self, ctx, mod_or_cmd: Optional[str]):
        pass

