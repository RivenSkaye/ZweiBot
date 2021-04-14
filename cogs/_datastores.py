""" Zwei's data handling module

A simple module that, depending on context, handles saving of data to
different data stores.
The choices are currently:
- SQLite3 DB, using Danny's async wrapper
- JSON files
- Plaintext key-value pairs
"""

import json
import asqlite
import os
from pathlib import Path

class datastore:
    pass

class jsonstore:
    pass

class sqlitestore:
    pass
