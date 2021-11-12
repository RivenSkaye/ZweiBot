from typing import Union, Dict, Optional, Any, List # Type hints
from abc import ABC, abstractmethod # Abstract Base Classes
from pathlib import Path # For loading files and taking in Path objects
import os # For OS-bound stuff like saving files
from sqlite3 import Row
import json
import asqlite as asql

class DataStore(ABC):
    """Object oriented approach for abstracting different types of datastores.

    There's only one argument required for instantiating any of the subclasses,
    this is the `Path` to the datastore, or the `str`ing representation of the
    path. As with any `Path` object or `str`ing equivalent, this may be both a
    relative or an absolute path, so long as it's correct.
    Subclasses may add additional arguments to instantiate them, for example
    dicts of args required for `asqlite` or the extra parameters for `json`.
    All keyword arguments other than the `Path` or `str` for the `store` kwarg
    will be consumed into a single `readopts` dict that is to be expanded as
    kwargs for whatever datastore is being used.

    This is a base class that defines several methods. The aim of it is to
    abstract different types of datastores like JSON, sqlite or csv away so
    that the main focus can be the application.
    The base class assumes it will be used and implemented asynchronously,
    hence why all methods are async.

    The use-case it's written for is a Discord bot, where depending on how
    much activity it sees, the requirements for data integrity and the
    performance impact of the size may differ depending on the host. That said,
    the concept can be useful in many other cases as well.

    For all of the functions/methods in this class, there are some typical
    datastore parameters that are required, though may be different depending
    on the backing engine. A list of these:
    - table: The table to call upon. For JSON this would be the top-level key,
    - key: the ID or key for the row to get. For JSON this is the 2nd level key,
        - For normal databases, this is probably some autogenerated value;
        - For text-based formats this might be a required field;
        - In order to make it not required, implement this base class with an
            auto-increment option or something similar;
    - data: the data to set or change for the selected row.

    This class is written with the intent of not raising during runtime.
    Instead, either make the error value in the return descriptive, or log the
    failure for debugging and make sure the application handles this correctly.
    The only exception is __init__, which may raise if a datastore can't be
    opened in the way requested by the user.
    """
    def __init__(self, store: Union[str,Path], openopts: Dict={"mode": "r"}, **readopts: Dict):
        self.file = store

    @abstractmethod
    async def get(self, table: str, key: Union[str,int]) -> Dict:
        """ Simple get function that returns a row or object from the DB.

        The return value is a dict containing the keys and values. If an
        implementation would rather return a list, either override this or
        shove the list into a dict.
        On errors, the error key should be set in the dict.
        """
        pass

    @abstractmethod
    async def set(self, table: str, data: Union[Dict,Any], key: Union[str,int]) -> bool:
        """ Function to add data to the DB. Make sure to handle caching!

        This function assumes the data is presented as a dict, containing
        a key set to the ID or key for the row and a value, or a set of values,
        for the row to be set.

        This is a blueprint function, so some implementations might use some
        form of caching like Redis or whatever MongoDB uses. Please do not
        defile my code by using it for Mongo though, it's bad and dangerous.

        The return value is `True` if successful, or `False` if not.
        """
        pass

    @abstractmethod
    async def update(self, table: str, data: Union[Dict,Any], key: Union[str,int]) -> bool:
        """ Update function for rows/entries in a datastore.

        The data object only has to contain the changed values, since dicts
        allow updating values in A only if they match keys in B. This means
        that the internal logic may very well just be
        `self.get(table,key).update(data)` followed by a saving it to disk.

        The return value should be the fully updated row, the fully updated
        datastore (for formats like JSON and csv) and it should have the error
        key set if something goes wrong.
        """
        pass

    @abstractmethod
    async def delete(self, table: str, key: Union[str,int]) -> Dict:
        """ Returns a value or row from the datastore and deletes it.

        This function first performs a `get` call to make sure the value exists
        and subsequently deletes the value from the datastore. For JSON objects
        this can be achieved using the `pop` method on the dict that represents
        it within the Python env. For most other datastores, this requires two
        operations, either one of which could theoretically fail.

        It is recommended to implement this function in the form of
        ```
        async def delete(self, table, key):
            val = self.get(table=table, key=key)
            <delete the value from the datastore in the proper manner>
            return val if val else {"error": "descriptive error message"}
        """
        pass

    @abstractmethod
    async def close(self) -> bool:
        """Close the database and disconnect if applicable for the datastore

        This method is purely for teardown purposes and any implementation of
        it should save the datastore if that hasn't been done during the last
        operation.
        """
        pass

    async def save(self) -> bool:
        """ Save the database to its file. This is optional to implement.

        Depending on the db system and config, caching or an in-memory object
        may be used for the datastore. This method is there to facilitate this
        by allowing either the `set` and `update` methods to save to disk, or
        by having this function implemented to have a caching system like
        Redis or similar save the datastore rather than keeping changes cached.

        Returns `True` when successful, or `False` if not so much.
        The default implementation will ALWAYS return false, since it assumes
        `update` and `set` do not depend on a caching mechanism.
        """
        print("Default implementations of DataStore assume `set` and `update` save changes to disk, or that some underlying mechanism will.")
        print("If changes after these methods were not saved to disk, blame whomever wrote the implementation of DataStore you're using.")
        return False

class JSONStore(DataStore):
    """ A basic `DataStore` implementation for working with JSON-based data

    This is written for data that doesn't require high integrity and that's
    not expected to become very large. For example the bot start config, that
    only ever needs to be read by the application, is good to have in JSON
    files so that it can be very quickly opened and read for data.

    Provides all methods synchronously as well, for convenience.
    """

    def __init__(self, store: Union[str,Path], openopts: Dict={"mode": "r"}, jsonopts: Dict={"indent": 4}, **readopts: Dict):
        """ Open a file and read it as JSON, with possible read and json options.

        This method adds `jsonopts` to the init signature, to add in options
        for the way the `json` module loads the file into a dict.
        This means all kwargs unknown to `__init__` will be passed to the `open`
        call to read the store, except for `store` which should be a `str` or
        `Path` and `jsonopts` which should be a `dict`. Both of these are
        entirely optional and use the libs' defaults.

        Changes made to the datastore using the set and update methods will be
        saved to disk immediately.

        Returns either the value of the requested entry, or None if it doesnt
        exist yet.
        """
        super().__init__(store=store)
        self.jsonopts = jsonopts
        try:
            with open(store, **openopts) as ds:
                self._store = json.load(ds)
        except Exception as ex:
            print(ex)
            raise ex

    def _save(self) -> bool:
        try:
            with open(self.file, mode="w") as s:
                json.dump(self._store, s, **self.jsonopts)
            return True
        except Exception as ex:
            print(ex)
            return False

    async def get(self, table: str, key: Union[str,int]) -> Dict:
        key = str(key)
        if table not in self._store: return {"error": f"The table `{table}` could not be found."}
        if key not in self._store[table]: return {"error": f"They key `{key}` does not exist in this document."}
        return {key: self._store[table][key]} # This might return {key: None} which can be valid in Python

    def get_sync(self, table: str, key: Union[str,int]) -> Dict:
        key = str(key)
        if table not in self._store: return {"error": f"The table `{table}` could not be found."}
        if key not in self._store[table]: return {"error": f"They key `{key}` does not exist in this document."}
        return {key: self._store[table][key]} # This might return {key: None} which can be valid in Python

    async def set(self, table: str, data: Any, key: Union[str,int]) -> bool:
        """ A function to add a single new key-value pair to the datastore.

        When adding one key, this function expects data to be any data type
        that is either a string, or which has a clear string representation.
        A single `v` value may be another dict for a JSON object.

        Returns True if successful, or False if not successful.
        When False is returned, check std.out for error information.
        """
        key = str(key)
        try:
            if table not in self._store: return {"error": f"The table `{table}` could not be found."}
            if not self._store[table][key]:
                self._store[table][key] = data
                return self._save()
            else:
                print("This value already exists, use the update function instead!")
                return False
        except Exception as ex:
            print(ex)
            return False

    def set_sync(self, table: str, data: Union[Dict,Any], key: Union[str,int]) -> bool:
        key = str(key)
        try:
            if table not in self._store: return {"error": f"The table `{table}` could not be found."}
            if not self._store[table][key]:
                self._store[table][key] = data
                return self._save()
            else:
                print("This value already exists, use the update function instead!")
                return False
        except Exception as ex:
            print(ex)
            return False

    async def update(self, table: str, data: Union[Dict,Any], key: Optional[Union[str,int]]=None) -> bool:
        """ Expects to either replace a key-value pair, or several of these.

        Expects to either update a single `key`, setting its value to `data`,
        or expects to update all of the keys in both `data` and `table` to the
        value listed for them in `data`.

        If `key` is supplied, it's value will be set to `data`, if `key` is not
        supplied, or explicitly set to None (the default value), the entire
        JSON dict will be updated and saved to disk.

        Also fails by returning False if any of the keys do not yet exist.
        """
        key = str(key)
        try:
            if not key is None:
                self._store[table][key] = data
                return self._save()
            else:
                for key in data.keys():
                    if not key in table.keys():
                        print(f"The key `{key}` does not exist, please use the set function for this")
                        return False
                self._store[table].update(data)
            return self._save()
        except Exception as ex:
            print(ex)
            return False

    def update_sync(self, table: str, data: Union[Dict,Any], key: Optional[Union[str,int]]=None) -> bool:
        key = str(key)
        try:
            if not key is None:
                self._store[table][key] = data
                return self._save()
            else:
                for key in data.keys():
                    if not key in table.keys():
                        print(f"The key `{key}` does not exist, please use the set function for this")
                        return False
                self._store[table].update(data)
            return self._save()
        except Exception as ex:
            print(ex)
            return False

    async def delete(self, table: str, key: Union[str,int]) -> Dict:
        val = self._store.pop(key, None)
        self._save()
        return val if val else {"error": "Either the key didn't exist, or it had a none value"}

    def delete_sync(self, table: str, key: Union[str,int]) -> Dict:
        val = self._store.pop(key, None)
        self._save()
        return val if val else {"error": "Either the key didn't exist, or it had a none value"}

    async def close(self) -> bool:
        return self._save() # Might as well save it an extra time

    def close_sync(self) -> bool:
        return self._save()

class SQLiteStore(DataStore):
    """ A `DataStore` implementation for use with Danny's `asqlite` library.

    This class assumes the dependency of `asqlite` has been met, this can be
    found at https://github.com/Rapptz/asqlite and all credit goes to the
    author Rapptz/Danny and the effort he made to create this library in order
    to use sqlite3 asynchronously.
    The **readopts remainder of kwargs will be ignored in this subclass, since
    a database initializes a connection object rather than opening a file in a
    reading and/or writing mode.
    """

    def __init__(self, store: Union[str,Path], openopts: Dict={}, **readopts: Dict):
        super().__init__(store=store)
        self._store = None
        self._openopts = openopts

    async def _conn(self):
        """ Initiates the actual connection, this is checked on every command.

        If anyone knows how to await shit in __init__, please tell us.
        """
        if self._store is None:
            self._store = await asql.connect(self.file, **self._openopts)

    async def get(self, table: str, key: Union[str,int], column: str="id") -> Dict:
        """ Simple get function to query the DB for data from specific tables.

        This implementation adds an optional parameter `column` to select rows
        based on a specific value for a column.
        The default column is `id`, the default behavior is to select all rows
        where the value of `column` equals `key`.

        Adds one key to the dict named `_SQLStoreInputQuery` for convenience.
        This may be used for debugging, logging the generated queries or any
        other use case that requires knowing what was passed to the database.

        This function assumes only one row is returned. When more than one row
        is expected, use `get_many` instead.

        The code makes sure to add quotes around names. But if there's any
        characters in the names that can break the query, you're on your own.
        Sanitizing user input does not fall within the scope of this class!

        For more complex `SELECT` statements, use `query_complex()`.
        The new parameter defaults to the row's `id` field.

        Returns the results of the first row matching the query.
        A query with no results is considered an error.
        """
        await self._conn()
        key = "'"+key+"'" if type(key) is str else key
        query = f"SELECT * FROM \"{table}\" WHERE \"{column}\"=={key};"
        res = await self._store.execute(query)
        fetch = await res.fetchone()
        if fetch is None:
            return {"error": f"The query did not match any results.",
                    "_SQLStoreInputQuery": query}
        keys = fetch.keys()
        retval = {}
        for key in keys:
            retval[key] = fetch[key]
        retval["_SQLStoreInputQuery"] = query
        return retval

    async def get_many(self, table: str, key: Union[List[Union[str,int]],Union[str,int]], column: str="id") -> List[Dict]:
        """ The same as get, but returns all results in a list.

        Takes a list of keys as well, for convenience. For using stuff like the
        SQL `NOT` operator, or `BETWEEN` or any other complex check, use the
        `query_complex()` function. It uses the `IN (keys)` operator to check
        for any values matching the keys. If only a single key is given and the
        field is unique, this is the same as `get()` except it returns a list.

        If a query only returns a single result, this function will still
        return a list containing that one entry.
        Might take a bit longer than `get` depending on the amount of results.
        """
        await self._conn()
        if type(key) is str:
            key = "'"+key+"'"
        elif type(key) is list:
            key = ",".join(key)
        query = f"SELECT * FROM \"{table}\" WHERE \"{column}\" IN ({key});"
        res = await self._store.execute(query)
        results = []
        fetch = await res.fetchone()
        if fetch is None:
            return [{"error": f"The query did not match any results.",
                    "_SQLStoreInputQuery": query}]
        keys = fetch.keys()
        # None evaluates to False. When the result list is over, the loop exits
        while fetch:
            val = {}
            for key in keys:
                val[key] = fetch[key]
            results.append(val)
            val["_SQLStoreInputQuery"] = query
            fetch = await res.fetchone()
        return results

    async def set(self, table: str, data: Dict, key: Union[str,int]=None) -> bool:
        """ Adds an entry to the given table, if it exists.

        If a table does not exist yet, create it through `query_complex()`.
        Make sure to add in any fields required for the given table. There is
        no guarantee that whatever problem arises is due to a malformed input.
        It might be because the requested table doesn't exist, or because a key
        was given that's illegal.
        Due to the way database systems work, the `key` argument defaults to
        `None` and will be ignored. If a specific key is required, add it in
        `data` instead. Usually, keys are fetched from other tables, or they
        are automatically generated.
        """
        await self._conn()
        columns = ",".join(data.keys())
        values = ",".join(data.values())
        query = f"INSERT OR FAIL INTO {table} ({columns}) VALUES ({values});"
        try:
            await self._store.execute(query)
            await self._store.commit()
            return True
        except Exception as ex:
            print(ex)
            return False

    async def update(self, table: str, data: Union[Dict,Any], key: Union[str,int], column: str="id") -> bool:
        """ Updates an entry in a table, if it exists.

        If a table doesn't exist yet, create it through `query_complex()`.
        Updates a row in a table using the key as an exact matching pattern for
        the specified column. By default, this is the id column for the table.
        """
        await self._conn()
        columns = ",".join(data.keys())
        values = ",".join(data.values())
        key = "'"+key+"'" if type(key) is str else key
        query = f"UPDATE OR FAIL {table} SET ({columns})=({values}) WHERE '{column}'=={key};"
        try:
            await self._store.execute(query)
            await self._store.commit()
            return True
        except Exception as ex:
            print(ex)
            return False

    async def delete(self, table: str, key: Union[str, int], column: str="id") -> Dict:
        """ Get and delete the requested row and return the value one last time.

        Internally calls `self.get(table=table, key=key, column=column)` before
        executing the DELETE statement, then commits the transaction and
        returns whatever the return value was for the `get` call
        """
        await self._conn()
        val = self.get(table=table, key=key, column=column)
        if not "error" in val.keys():
            # If it didn't error, it exists and we delete it.
            query = f"DELETE FROM {table} WHERE {column} == {key}"
            self._store.execute(query)
            self._store.commit()
            val["_SQLStoreInputQuery"] = query
        return val

    async def close(self) -> bool:
        """ Commit and close the database connection.
        """
        if self._store is None:
            return True
        try:
            await self._store.commit() # Make sure we save
            await self._store.close()
        except Exception as ex:
            print(ex)
            return False
        return True


    async def query_complex(self, query: str, commit: bool=False) -> Union[Row,Dict]:
        """ A simple wrapper that executes a given query, sanitize user input!

        This class is literally just going to execute whatever query it gets,
        without attempting to sanitize any input or whatever. This is only
        provided for edgecases where there needs to be more freedom to change
        the database from the code.
        It takes one parameter besides the query; a boolean whether or not it
        should call `commit` on the connection object. This defaults to False
        as the function should
        """
        await self._conn()
        try:
            await self._store.execute(query)
            return self._store.commit() if commit else True
        except Exception as ex:
            print(ex)
            return {"error": ex, "_SQLStoreInputQuery": query}