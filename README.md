# Zwei

This Discord bot is the successor to my previous project, Prysm.
After attempting to rewrite that in proper code, I hit a brick wall in moving
over the code from its monolithic structure to a proper division of classes and
cogs. Trying to change it kinda killed my motivation and I ended up deciding that
if I was gonna do it properly anyway, I might as well rewrite it completely.

So here's Zwei. A new identity, a new face and a fresh start. The only thing
she inherits from Prysm is the knowledge I gained creating and partially
porting over the code.

And now, here we stand with yet another rewrite. In a different language this time.
After Danny ceased development on discord.py, there were a few devs offering
an alternative that wouldn't need any changes from existing code other than what
had changed with dpy 2.0.0a. Well, either Danny started writing code that was
way different from what he used to write, or their promises of not changing the
API too much were bogus.
So in the vein of what this bot was meant for, learning a new lang, I'm now
using it to teach myself Rust. And I gotta say I like it!

## The Goal

Zwei was created with the idea of making a bot that anyone can host and edit
to meet their needs, but there is some basic functionality I want to provide
for the bot in order to have a solid base to work off of.

The goal for Zwei is to be a fully featured bot to suit all basic server needs.
She's there as a support for mods and admins, sending people to Lost Blue if
they misbehave (banning them), or just temporarily removing them from the server.
In the near future, she'll also aim to help people set up a server by allowing
them easy channel and role creation, editing their settings, etc.
Besides that, she'll also offer changing nicknames when the display name isn't
what the rules would allow and she'd be happy to help make announcements through
a central place, mention roles in certain contexts where a role should not be
mentionable for everyone, as well as making sure that this system won't be abused
by users to make these mentions unnecessarily. All in all, Zwei's core modules
exist to make server management and moderation as easy as possible.

_Other public features are yet to be determined and up to the whims of the devs._

# Owner-only Features

Some features would be very intensive to host for more than a couple of servers
that a basement computer can support. These features would require people to
host their own Zwei\* to use these features in their own servers.

Zwei can build a database of music for streaming, so long as files are provided
that she can access normally. This means direct access links to files, links to
files hosted on services like fiery or pomf, or even system local files.
So long as `ffmpeg` can read and transcode it for use with Discord, Zwei will be
happy to stream it into any voice channel she can access.

_No guarantees will be made regarding the audio quality when transcoding._
