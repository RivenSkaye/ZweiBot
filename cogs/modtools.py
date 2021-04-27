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

    @commands.Command(name="kick", brief="Kicks a user from the server", aliases=["remove","prod","eject"])
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
