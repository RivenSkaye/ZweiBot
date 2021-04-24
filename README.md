# Zwei

This Discord bot is the successor to my previous project, Prysm.
After attempting to rewrite that in proper code, I hit a brick wall in moving
over the code from its monolithic structure to a proper division of classes and
cogs. Trying to change it kinda killed my motivation and I ended up deciding that
if I was gonna do it properly anyway, I might as well rewrite it completely.

So here's Zwei. A new identity, a new face and a fresh start. The only thing
she inherits from Prysm is the knowledge I gained creating and partially
porting over the code. And I'm also applying the knowledge I gained by helping
out other bot devs with some stuff when I lost motivation for my own work.

## The Goal

Zwei was created with the idea of making a bot that anyone can host and edit
to meet their needs, but there is some basic functionality I want to provide
for the bot in order to have a solid base to work off of.

The goal for Zwei is to be a fully featured bot to suit all basic server needs.
She aims to be able to help people set up a server by changing channel topics,
setting permissions, managing roles and making it easier to create or delete
them, as well as setting a color for them. Changing nicknames when the display
name isn't what the rules would like to see or allow, kick and ban members that
violate server rules, help make announcements through a central place, mention
roles in certain contexts where a role should not be mentionable for everyone,
as well as making sure that this system won't be abused by users to make these
mentions unnecessarily. Delete and purge messages from the history if stuff
happens that shouldn't remain and providing an easy way of muting / deafening /
moving members when they misbehave in a voice channel.

All in all, Zwei's core modules exist to make server management and moderation
as easy as possible, with a few bonuses.

## The Bonus Features

As with a lot of bots, Zwei is a project of passion and thus some extra features
are added to her arsenal of commands.

Zwei will have a set of extra features, like action and reaction GIFs, reminders,
repeating messages and following RSS feeds. Naturally a list will be made and
updated here as these features get implemented.

# Owner-only Features

Some features would be very intensive to host for more than a couple of servers
that a basement computer can support. These features would require people to
host their own Zwei\* to use these features in their own servers.

Zwei can build a database of music for streaming, so long as files are provided
that she can access normally. This means direct access links to files, links to
files hosted on services like fiery or pomf, or perhaps even local files.
So long as `ffmpeg` can read and transcode it for use with Discord, Zwei will be
happy to stream it into any voice channel she can access.

# Self-hosting

_Zwei's code is freely available, the image used for her avatar is not included
since I asked the artist who made it for permission to use it. If you wish to
host your own Zwei, find a different image and preferably use a different name
for the bot as well._

The act of self-hosting a bot is actually nothing more than running the code
that makes it work on your own VPS or computer. It's recommended to use either
a "basement PC" that isn't actively in use by people so as not to interfere with
normal computer usage, or to look into VPS/hosting solutions with a third party.

_The Zwei developers are not responsible for these things and we can't help you
get set up with either of these systems. Though the support server (once there
is one) may give out advice and personal recommendations, you are liable for
your own actions. The code is provided as-is and a list of dependencies is
[included in this file](#Dependencies). No guarantee is given that it may work
and none of the people responsible for developing this code can be held
responsible for any damages, breakage or mistakes made._

In order to self-host Zwei there are a few dependencies that need to be met.
These dependencies are/will be at least:

- Python version 3.5.3 or higher (for discord.py);
- [`discord.py`](https://github.com/Rapptz/discord.py) and all of its dependencies
	- Please note that when using music or voice modules,
	[`discord.py[voice]`](https://github.com/Rapptz/discord.py) is required instead;
	- As well as [ffmpeg](https://ffmpeg.org/) for the music components.
- [`apscheduler`](https://apscheduler.readthedocs.io/en/stable/index.html) for
remind and repeat functionality;
- [sqlite3](https://sqlite.org/index.html) for databases (optional);
- [`asqlite`](https://github.com/Rapptz/asqlite) for the Database code (optional);
