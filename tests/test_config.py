#!/usr/bin/env python3
#
# Copyright (c) 2020 Miklos Vajna and contributors.
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

"""The test_config module covers the config module."""

from typing import Dict
from typing import List
from typing import Optional
import calendar
import datetime
import os

import config


def make_test_config() -> config.Config:
    """Creates a Config instance that has its root as /tests."""
    return config.Config("tests")


class TestFileSystem(config.FileSystem):
    """File system implementation, for test purposes."""
    def __init__(self) -> None:
        self.__hide_paths: List[str] = []
        self.__mtimes: Dict[str, float] = {}

    def set_hide_paths(self, hide_paths: List[str]) -> None:
        """Sets the hide paths."""
        self.__hide_paths = hide_paths

    def set_mtimes(self, mtimes: Dict[str, float]) -> None:
        """Sets the mtimes."""
        self.__mtimes = mtimes

    def path_exists(self, path: str) -> bool:
        if path in self.__hide_paths:
            return False
        return os.path.exists(path)

    def getmtime(self, path: str) -> float:
        if path in self.__mtimes:
            return self.__mtimes[path]
        return os.path.getmtime(path)


class URLRoute:
    """Contains info about how to patch out one URL."""
    # The request URL
    url: str
    # Path of expected POST data, empty for GET
    data_path: str
    # Path of expected result data
    result_path: str

    def __init__(self, url: str, data_path: str, result_path: str) -> None:
        self.url = url
        self.data_path = data_path
        self.result_path = result_path


class TestNetwork(config.Network):
    """Network implementation, for test purposes."""
    def __init__(self, routes: List[URLRoute]) -> None:
        self.__routes = routes

    def urlopen(self, url: str, data: Optional[bytes] = None) -> bytes:
        for route in self.__routes:
            if url != route.url:
                continue

            if route.data_path:
                with open(route.data_path, "rb") as stream:
                    expected = stream.read()
                    if data != expected:
                        assert data
                        assert data == expected, \
                            "bad data: actual is '" + str(data, 'utf-8') + \
                            "', expected '" + str(expected, "utf-8") + "'"

            with open(route.result_path, "rb") as stream:
                return stream.read()

        assert False, "url missing from route list: '" + url + "'"


class TestTime(config.Time):
    """Time implementation, for test purposes."""
    def __init__(self, now: float) -> None:
        self.__now = now

    def now(self) -> float:
        return self.__now


def make_test_time() -> config.Time:
    """Generates unix timestamp for 2020-05-10."""
    return TestTime(calendar.timegm(datetime.date(2020, 5, 10).timetuple()))
