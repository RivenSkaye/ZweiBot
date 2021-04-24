from discord.ext import commands

class ModTools(commands.Cog, name="Moderation"):
    """ A set of tools to aid in server moderation.

    This cog contains server moderation commands that automate some of the work
    that moderators would have to do otherwise.
    It does require the user using the commands to have the corresponding
    permissions in this server.
    """
    def __init__(self, bot):
        self.bot = bot

    @commands.Command()
