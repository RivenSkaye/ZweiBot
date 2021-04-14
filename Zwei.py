""" Zwei, the Discord utility bot

This is the core module, it contains base functions used throughout the bot
and performs checks on the config for which modules to load. It also contains
a few functions that proxy commands to different optional modules, like using
JSON or SQLite for data storage and such.

Author: Riven Skaye
"""
# Type hints and function signatures
from typing import List, Dict, Union, Any
from collections.abc import Callable
# Basic functioning
import os
from pathlib import Path
import asyncio
# Discord stuff
import discord
from discord.ext import commands
# For timed stuff
from apscheduler.schedulers.asyncio import AsyncIOScheduler
from apscheduler.job import Job

class ZweiBot(commands.AutoShardedBot):
    pass
