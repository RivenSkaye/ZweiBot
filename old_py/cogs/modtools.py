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
        """ Deletes the last x messages, 5 by default. Max 250 at a time!

        Performs a check, if any of the messages are pinned, they're not
        deleted. They do still count towards the 250 total.
        If the messages aren't pinned, they're deleted if the user calling
        the command has all proper permissions.
        """
        def _is_pinned(msg) -> bool:
            return not msg.pinned
        if amount < 1:
            await ctx.send("Could you stop trying to purge thin air?")
            return
        elif amount > 250:
            await ctx.send("Please keep the amount of messages to be purged somewhat manageable.\nNo more than 250 at a time, okay?")
            return
        actual_amount = amount + 1
        remainder = actual_amount
        while remainder > 0:
            limit = 100 if remainder > 99 else remainder
            await ctx.channel.purge(limit=limit, bulk=True, oldest_first=False, check=_is_pinned)
            remainder = remainder - limit
            if remainder > 0:
                asyncio.sleep(0.25)
        # We all have that one line where we pretend PEP8 does not exist
        await ctx.send(f"I've deleted the last {amount} messages for you, except for those that were pinned." if amount > 1 else "I've deleted the last message, unless it was pinned. _You could've done that faster manually._")

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