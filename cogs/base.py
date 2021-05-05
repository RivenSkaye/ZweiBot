from discord.ext import commands
from typing import Optional
# Uptime
from datetime import datetime
from asyncio import sleep

class BaseCommands(commands.Cog, name="Base"):
    """ Base commands for use with Zwei

    These are the base commands that prove she works as intended.
    Anything else that breaks can be lost for a while, but these commands are
    paramount for both using and testing basic stuff.
    """
    def __init__(self, bot):
        self.bot = bot

    @commands.command(name="uptime", brief="List uptime for the bot", hidden=True)
    @commands.is_owner()
    async def uptime(self, ctx):
        """Lists total uptime for the bot, in a human readable format.

        Lists the time as `xx days, hh:mm:ss`. This can be used indicatively
        for debugging purposes (for example long term usage filling up memory)
        and as a gimmick for the owners. Honestly, it's just here to test the
        decorators and to check if the bot functions at all.
        """
        uptime = str(datetime.utcnow() - self.bot.starttime)
        await ctx.send(f"I've been giving it my all for {uptime} now!")

    @commands.command(name="prefix")
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
        server_prefix = await self.bot._config.get(table="prefixes", key=key)
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
                success = await self.bot._config.set(table="prefixes", data=new_prefix, key=key)
            else:
                success = await self.bot._config.update(table="prefixes", data=new_prefix, key=key)

            if success:
                new_prefix = new_prefix.replace("`", "\`")
                await ctx.send(f"Prefix changed to `{new_prefix}`")
            else:
                await ctx.send("I couldn't change the prefix, something went wrong!\nContact the devs for assistance")

    @commands.command(name="shutdown", aliases=["exit", "panic", "die", "sleep"])
    @commands.is_owner()
    async def shutdown(self, ctx, time: int=1):
        """ Shuts down after optionally <time> seconds. Owner only.

        If you ever see this getting called, wrap up your shit and exit.
        """
        timetxt = f"{time} seconds" if time > 1 else "1 second"
        await ctx.reply(f"I need some rest, so I'll give you {timetxt} before I go and take a nap")
        await sleep(time if time > 0 else 1)
        await self.bot.close()

    @commands.command(name="say")
    async def say(self, ctx, *, text: str, embed: Optional[str]=None):
        """ Make the bot repeat what you say. Embeds don't work yet.

        Parse the user input and check for any role mentions and the if the
        user has the appropriate permissions to actually mention them. If the
        user is adding the role mention into an embed, let it be since that's
        not going to send a notification to anyone anyways.
        """
        await ctx.send(text)

    @commands.command(name="load")
    @commands.is_owner()
    async def load_cog(self, ctx, cog: str):
        """ Loads a cog by name, if it exists.

        Performs a lookup in the cogs folder to see if the requested cog name exists.
        Then proceeds to load the cog if it does, or it sends an error that it doesn't.
        """
        try:
            self.bot.load_extension(f"cogs.{cog}")
            await ctx.reply(f"Successfully loaded _{cog}_.")
        except Exception as ex:
            await ctx.reply("Something went wrong trying to load _{cog}_.\n{ex}")

    @commands.command(name="reload")
    @commands.is_owner()
    async def reload_cog(self, ctx, cog: str):
        """Reloads a loaded cog. Useful for code changes.

        This can be used when working on development, or when a cog misbehaves and
        needs to be reinitialized. Expect to use this for debugging as well.
        """
        try:
            self.bot.reload_extension(f"cogs.{cog}")
            await ctx.reply(f"Successfully reloaded _{cog}_.")
        except Exception as ex:
            await ctx.reply("Something went wrong trying to reload _{cog}_.\n{ex}")

    @commands.command(name="unload")
    @commands.is_owner()
    async def unload_cog(self, ctx, cog: str):
        """Unloads a loaded cog. Useful for code changes.

        Yeets the cog from the bot until you load it again.
        """
        try:
            self.bot.unload_extension(f"cogs.{cog}")
            await ctx.reply(f"Successfully unloaded _{cog}_.")
        except Exception as ex:
            await ctx.reply("Get rekt, this is so broken I can't yeet it. Something went wrong trying to unload _{cog}_.\n{ex}")

def setup(bot):
    bot.add_cog(BaseCommands(bot))
