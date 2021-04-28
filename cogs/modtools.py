from discord.ext import commands
import discord

from typing import Union

class ModTools(commands.Cog, name="Moderation"):
    """ A set of tools to aid in server moderation.

    This cog contains server moderation commands that automate some of the work
    that moderators would have to do otherwise.
    It does require the user using the commands to have the corresponding
    permissions in this server.
    """
    def __init__(self, bot):
        self.bot = bot

    @commands.command(name="purge", aliases=["prune","massdelete","massdel"])
    @commands.has_permissions(manage_messages=True)
    @commands.guild_only()
    async def purge(self, ctx, amount: int=5):
        if amount < 1:
            await ctx.send("Could you stop trying to purge thin air?")
            return
        elif amount > 250:
            await ctx.send("Please keep the amount of messages to be purged somewhat manageable.\nNo more than 250 at a time, okay?")
            return
        remainder = amount
        while remainder > 0:
            limit = 100 if remainder > 99 else remainder
            await ctx.channel.purge(limit=limit, bulk=True, oldest_first=True)
            remainder = remainder - limit
            if remainder > 0:
                asyncio.sleep(0.25)
        await ctx.send(f"I deleted the last {amount} of messages for you." if amount > 1 else "I deleted the last message. _You could've done that faster manually._")

    @commands.command(name="kick", brief="Kicks a user from the server", aliases=["remove","prod","eject"])
    @commands.has_permissions(kick_members=True)
    @commands.guild_only()
    async def kick(self, ctx, user: Union[discord.Member,int], *, reason: str="You broke a rule."):
        """ Kicks the user and notifies them why, if a reason was given.

        Usage: `;kick <@user> [bruh, you broke rule 34!]` or `;kick <User_ID> [bad behavior.]`
        Sends the user a message in the format of `You were kicked from {guild.name}\nMessage: {reason}`.
        If a reason is not given, it defaults to "You broke a rule."
        """
        user = await ctx.guild.get_member(user) if isinstance(user, int) else user
        kickmsg = f"I removed {user.name} from the guild. Make sure to keep an eye on them if they return here."
        if not user.dm_channel:
            await user.create_dm()
        await user.kick()
        await ctx.send(kickmsg)

def setup(bot):
    bot.add_cog(ModTools(bot))
