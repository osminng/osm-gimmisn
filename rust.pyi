#!/usr/bin/env python3
#
# Copyright (c) 2021 Miklos Vajna and contributors.
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

"""
Type hints for rust.so.
"""

from typing import Any
from typing import BinaryIO
from typing import Dict
from typing import List
from typing import Optional
from typing import Set
from typing import Tuple
from typing import TypeVar
from typing import cast
import api


class PyDoc:
    """Generates xml/html documents."""
    def __init__(self) -> None:
        ...

    def get_value(self) -> str:
        """Gets the escaped value."""
        ...


class PyStdFileSystem(api.FileSystem):
    """File system implementation, backed by the Rust stdlib."""
    def __init__(self) -> None:
        ...

    def path_exists(self, path: str) -> bool:
        ...

    def getmtime(self, path: str) -> float:
        ...

    def open_read(self, path: str) -> BinaryIO:
        ...

    def open_write(self, path: str) -> BinaryIO:
        ...

class PyIni:
    """Configuration file reader."""
    def __init__(self, config_path: str, root: str) -> None:
        ...

    def get_workdir(self) -> str:
        """Gets the directory which is writable."""
        ...

    def get_uri_prefix(self) -> str:
        """Gets the global URI prefix."""
        ...

class PyContext:
    """Context owns global state which is set up once and then read everywhere."""
    def __init__(self, prefix: str) -> None:
        ...

    def get_abspath(self, rel_path: str) -> str:
        """Make a path absolute, taking the repo root as a base dir."""
        ...

    def get_ini(self) -> PyIni:
        """Gets the ini file."""
        ...

    def set_network(self, network: api.Network) -> None:
        """Sets the network implementation."""
        ...

    def get_network(self) -> api.Network:
        """Gets the network implementation."""
        ...

    def set_time(self, time: api.Time) -> None:
        """Sets the time implementation."""
        ...

    def get_time(self) -> api.Time:
        """Gets the time implementation."""
        ...

    def set_subprocess(self, subprocess: api.Subprocess) -> None:
        """Sets the subprocess implementation."""
        ...

    def get_subprocess(self) -> api.Subprocess:
        """Gets the subprocess implementation."""
        ...

    def set_unit(self, unit: api.Unit) -> None:
        """Sets the unit implementation."""
        ...

    def get_unit(self) -> api.Unit:
        """Gets the unit implementation."""
        ...

    def set_file_system(self, file_system: api.FileSystem) -> None:
        """Sets the file system implementation."""
        ...

    def get_file_system(self) -> api.FileSystem:
        """Gets the file system implementation."""
        ...

def py_get_content(path: str) -> bytes:
    """Gets the content of a file in workdir."""
    ...

class PyRelationFiles:
    """A relation's file interface provides access to files associated with a relation."""
    def __init__(self, workdir: str, name: str):
        ...

    def get_ref_streets_path(self) -> str:
        """Build the file name of the reference street list of a relation."""
        ...

    def get_osm_streets_path(self) -> str:
        """Build the file name of the OSM street list of a relation."""
        ...

    def get_osm_housenumbers_path(self) -> str:
        """Build the file name of the OSM house number list of a relation."""
        ...

    def get_ref_housenumbers_path(self) -> str:
        """Build the file name of the reference house number list of a relation."""
        ...

    def get_housenumbers_percent_path(self) -> str:
        """Builds the file name of the house number percent file of a relation."""
        ...

    def get_housenumbers_htmlcache_path(self) -> str:
        """Builds the file name of the house number HTML cache file of a relation."""
        ...

    def get_streets_percent_path(self) -> str:
        """Builds the file name of the street percent file of a relation."""
        ...

    def get_streets_additional_count_path(self) -> str:
        """Builds the file name of the street additional count file of a relation."""
        ...

    def get_housenumbers_additional_count_path(self) -> str:
        """Builds the file name of the housenumber additional count file of a relation."""
        ...

    def get_ref_streets_read_stream(self, ctx: PyContext) -> BinaryIO:
        """Opens the reference street list of a relation for reading."""
        ...

    def get_osm_streets_read_stream(self, ctx: PyContext) -> BinaryIO:
        """Opens the OSM street list of a relation for reading."""
        ...

    def get_osm_housenumbers_read_stream(self, ctx: PyContext) -> BinaryIO:
        """Opens the OSM house number list of a relation for reading."""
        ...

    def get_ref_housenumbers_read_stream(self, ctx: PyContext) -> BinaryIO:
        """Opens the reference house number list of a relation for reading."""
        ...

    def get_streets_percent_read_stream(self, ctx: PyContext) -> BinaryIO:
        """Opens the street percent file of a relation for reading."""
        ...

    def get_streets_additional_count_write_stream(self, ctx: PyContext) -> BinaryIO:
        """Opens the street additional count file of a relation for writing."""
        ...

    def get_housenumbers_additional_count_write_stream(self, ctx: PyContext) -> BinaryIO:
        """Opens the housenumbers additional count file of a relation for writing."""
        ...

    def write_osm_streets(self, ctx: PyContext, result: str) -> int:
        """Writes the result for overpass of Relation.get_osm_streets_query()."""
        ...

    def write_osm_housenumbers(self, ctx: PyContext, result: str) -> int:
        """Writes the result for overpass of Relation.get_osm_housenumbers_query()."""
        ...

class PyRelationConfig:
    """A relation configuration comes directly from static data, not a result of some external query."""
    def set_active(self, active: bool) -> None:
        """Sets if the relation is active."""
        ...

    def is_active(self) -> bool:
        """Gets if the relation is active."""
        ...

    def get_osmrelation(self) -> int:
        """Gets the OSM relation object's ID."""
        ...

    def get_refcounty(self) -> str:
        """Gets the relation's refcounty identifier from reference."""
        ...

    def get_refsettlement(self) -> str:
        """Gets the relation's refsettlement identifier from reference."""
        ...

    def get_alias(self) -> List[str]:
        """Gets the alias(es) of the relation: alternative names which are also accepted."""
        ...

    def set_housenumber_letters(self, housenumber_letters: bool) -> None:
        """Sets the housenumber_letters property from code."""
        ...

    def get_refstreets(self) -> Dict[str, str]:
        """Returns an OSM name -> ref name map."""
        ...

    def set_filters(self, filters: str) -> None:
        """Sets the 'filters' key from code."""
        ...

    def get_filters(self) -> Optional[str]:
        """Returns a street name -> properties map."""
        ...

    def get_street_refsettlement(self, street: str) -> List[str]:
        """Returns a list of refsettlement values specific to a street."""
        ...

    def get_street_filters(self) -> List[str]:
        """Gets list of streets which are only in reference, but have to be filtered out."""
        ...

    def get_osm_street_filters(self) -> List[str]:
        """Gets list of streets which are only in OSM, but have to be filtered out."""
        ...

class PyRelation:
    """A relation is a closed polygon on the map."""
    def get_name(self) -> str:
        """Gets the name of the relation."""
        ...

    def get_files(self) -> PyRelationFiles:
        """Gets access to the file interface."""
        ...

    def get_config(self) -> PyRelationConfig:
        """Gets access to the config interface."""
        ...

    def set_config(self, config: PyRelationConfig) -> None:
        """Sets the config interface."""
        ...

    def get_osm_streets_query(self) -> str:
        """Produces a query which lists streets in relation."""
        ...

    def get_osm_housenumbers_query(self) -> str:
        """Produces a query which lists house numbers in relation."""
        ...

    def get_invalid_refstreets(self) -> Tuple[List[str], List[str]]:
        """Returns invalid osm names and ref names."""
        ...

class PyRelations:
    """A relations object is a container of named relation objects."""
    def __init__(self, ctx: PyContext) -> None:
        ...

    def get_workdir(self) -> str:
        """Gets the workdir directory path."""
        ...

    def get_relation(self, name: str) -> PyRelation:
        """Gets the relation that has the specified name."""
        ...

    def set_relation(self, name: str, relation: PyRelation) -> None:
        """Sets a relation for testing."""
        ...

    def get_names(self) -> List[str]:
        """Gets a sorted list of relation names."""
        ...

    def get_active_names(self) -> List[str]:
        """Gets a sorted list of active relation names."""
        ...

    def get_relations(self) -> List[PyRelation]:
        """Gets a list of relations."""
        ...

def py_get_request_uri(environ: Dict[str, str], ctx: PyContext, relations: PyRelations) -> str:
    """Finds out the request URI."""
    ...

def py_handle_no_osm_housenumbers(prefix: str, relation_name: str) -> PyDoc:
    """Handles the no-osm-housenumbers error on a page using JS."""
    ...

def py_handle_no_ref_housenumbers(prefix: str, relation_name: str) -> PyDoc:
    """Handles the no-ref-housenumbers error on a page using JS."""
    ...

def py_handle_github_webhook(data: bytes, ctx: PyContext) -> PyDoc:
    """Handles a GitHub style webhook."""
    ...

def py_is_missing_housenumbers_txt_cached(ctx: PyContext, relation: PyRelation) -> bool:
    """Decides if we have an up to date plain text cache entry or not."""
    ...

def py_get_missing_housenumbers_txt(ctx: PyContext, relation: PyRelation) -> str:
    """Gets the cached plain text of the missing housenumbers for a relation."""
    ...

def py_handle_main_housenr_additional_count(ctx: PyContext, relation: PyRelation) -> PyDoc:
    """Handles the housenumber additional count part of the main page."""
    ...

def py_application(
        request_headers: Dict[str, str],
        request_data: bytes,
        ctx: PyContext
) -> Tuple[str, List[Tuple[str, str]], bytes]:
    """The entry point of this WSGI app."""
    ...

def py_get_topcities(ctx: PyContext, src_root: str) -> List[Tuple[str, int]]:
    """
    Generates a list of cities, sorted by how many new hours numbers they got recently.
    """
    ...

# vim:set shiftwidth=4 softtabstop=4 expandtab:
