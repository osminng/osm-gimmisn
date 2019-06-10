#!/usr/bin/env python3
#
# Copyright (c) 2019 Miklos Vajna and contributors.
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

"""The helpers module contains functionality shared between other modules."""

import re
import os
import pickle
from typing import Callable, Dict, Iterable, List, Sequence, Tuple
import yaml


class Range:
    """A range object represents an odd or even range of integer numbers."""
    def __init__(self, start, end):
        self.start = start
        self.end = end
        self.is_odd = start % 2 == 1

    def __contains__(self, item):
        if self.is_odd != (item % 2 == 1):
            return False
        if self.start <= item <= self.end:
            return True
        return False

    def __repr__(self):
        return "Range(start=%s, end=%s, is_odd=%s)" % (self.start, self.end, self.is_odd)

    def __eq__(self, other):
        if self.start != other.start:
            return False
        if self.end != other.end:
            return False
        return True


class Ranges:
    """A Ranges object contains an item if any of its Range objects contains it."""
    def __init__(self, items):
        self.items = items

    def __contains__(self, item):
        for i in self.items:
            if item in i:
                return True
        return False

    def __repr__(self):
        return "Ranges(items=%s)" % self.items

    def __eq__(self, other):
        return self.items == other.items


def get_reftelepules_list_from_yaml(reftelepules_list, value):
    """Determines street-level and range-level reftelepules overrides."""
    if "reftelepules" in value.keys():
        reftelepules_list = [value["reftelepules"]]
    if "ranges" in value.keys():
        for street_range in value["ranges"]:
            if "reftelepules" in street_range.keys():
                reftelepules_list.append(street_range["reftelepules"])

    return reftelepules_list


def get_street_details(datadir, street, relation_name):
    """Determines the ref codes, street name and type for a street in a relation."""
    with open(os.path.join(datadir, "relations.yaml")) as sock:
        relations = yaml.load(sock)
    relation = relations[relation_name]
    refmegye = relation["refmegye"]
    reftelepules_list = [relation["reftelepules"]]

    refstreets = {}  # type: Dict[str, str]
    if os.path.exists(os.path.join(datadir, "housenumber-filters-%s.yaml" % relation_name)):
        with open(os.path.join(datadir, "housenumber-filters-%s.yaml" % relation_name)) as sock:
            # See if config wants to map:
            root = yaml.load(sock)
            if "refstreets" in root.keys():
                # From OSM name to ref name.
                refstreets = root["refstreets"]
            if "filters" in root.keys():
                # street-specific reftelepules override.
                filters = root["filters"]
                for filter_street, value in filters.items():
                    if filter_street == street:
                        reftelepules_list = get_reftelepules_list_from_yaml(reftelepules_list, value)

    if street in refstreets.keys():
        street = refstreets[street]

    tokens = street.split(' ')
    street_name = " ".join(tokens[:-1])
    street_type = tokens[-1]
    return refmegye, sorted(set(reftelepules_list)), street_name, street_type


def sort_numerically(strings: Iterable[str]) -> List[str]:
    """Sorts strings according to their numerical value, not alphabetically."""
    return sorted(strings, key=split_house_number)


def split_house_number(house_number: str) -> Tuple[int, str]:
    """Splits house_number into a numerical and a remainder part."""
    match = re.search(r"^([0-9]*)([^0-9].*|)$", house_number)
    if not match:  # pragma: no cover
        return (0, '')
    number = 0
    try:
        number = int(match.group(1))
    except ValueError:
        pass
    return (number, match.group(2))


def sort_streets_csv(data: str) -> str:
    """
    Sorts TSV Overpass street name result with visual partitioning.

    See split_street_line for sorting rules.
    """
    return process_csv_body(sort_streets, data)


def sort_streets(lines: Iterable[str]) -> List[str]:
    """
    Sorts the body of a TSV Overpass street name result with visual partitioning.

    See split_street_line for sorting rules.
    """
    return sorted(lines, key=split_street_line)


def split_street_line(line: str) -> Tuple[bool, str, str, str, Tuple[int, str]]:
    """
    Augment TSV Overpass street name result lines to aid sorting.

    It prepends a bool to indicate whether the street is missing a name, thus
    streets with missing names are ordered last.
    oid is interpreted numerically while other fields are taken alphabetically.
    """
    field = line.split('\t')
    oid = get_array_nth(field, 0)
    name = get_array_nth(field, 1)
    highway = get_array_nth(field, 2)
    service = get_array_nth(field, 3)
    missing_name = name == ''
    return (missing_name, name, highway, service, split_house_number(oid))


def process_csv_body(fun: Callable[[Iterable[str]], List[str]], data: str) -> str:
    """
    Process the body of a CSV/TSV with the given function while keeping the header intact.
    """
    lines = data.split('\n')
    header = lines[0] if lines else ''
    body = lines[1:] if lines else ''
    result = [header] + fun(body)
    return '\n'.join(result)


def sort_housenumbers_csv(data: str) -> str:
    """
    Sorts TSV Overpass house numbers result with visual partitioning.

    See split_housenumber_line for sorting rules.
    """
    return process_csv_body(sort_housenumbers, data)


def sort_housenumbers(lines: Iterable[str]) -> List[str]:
    """
    Sorts the body of a TSV Overpass house numbers result with visual partitioning.

    See split_housenumber_line for sorting rules.
    """
    return sorted(lines, key=split_housenumber_line)


def split_housenumber_line(line: str) -> Tuple[str, bool, bool, str, Tuple[int, str], str,
                                               Tuple[int, str], Iterable[str], Tuple[int, str]]:
    """
    Augment TSV Overpass house numbers result lines to aid sorting.

    It prepends two bools to indicate whether an entry is missing either a house number, a house name
    or a conscription number.
    Entries lacking either a house number or all of the above IDs come first.
    The following fields are interpreted numerically: oid, house number, conscription number.
    """
    field = line.split('\t')

    oid = get_array_nth(field, 0)
    street = get_array_nth(field, 1)
    housenumber = get_array_nth(field, 2)
    postcode = get_array_nth(field, 3)
    housename = get_array_nth(field, 4)
    cons = get_array_nth(field, 5)
    tail = field[6:] if len(field) > 6 else []

    have_housenumber = housenumber != ''
    have_houseid = have_housenumber or housename != '' or cons != ''
    return (postcode, have_houseid, have_housenumber, street,
            split_house_number(housenumber),
            housename, split_house_number(cons), tail, split_house_number(oid))


def get_array_nth(arr: Sequence[str], index: int) -> str:
    """Gets the nth element of arr, returns en empty string on error."""
    return arr[index] if len(arr) > index else ''


def get_only_in_first(first, second):
    """Returns items which are in first, but not in second."""
    ret = []
    for i in first:
        if i not in second:
            ret.append(i)
    return ret


def get_in_both(first, second):
    """Returns items which are in both first and second."""
    ret = []
    for i in first:
        if i in second:
            ret.append(i)
    return ret


def git_link(version: str, prefix: str) -> str:
    """Generates a HTML link based on a website prefix and a git-describe version."""
    commit_hash = re.sub(".*-g", "", version)
    return "<a href=\"" + prefix + commit_hash + "\">" + version + "</a>"


def get_nth_column(path: str, column: int) -> List[str]:
    """Reads the content of path, interprets its content as tab-separated values, finally returns
    the values of the nth column. If a row has less columns, that's silentely ignored."""
    ret = []

    with open(path) as sock:
        first = True
        for line in sock.readlines():
            if first:
                first = False
                continue

            tokens = line.strip().split('\t')
            if len(tokens) < column + 1:
                continue

            ret.append(tokens[column])

    return ret


def get_streets(workdir: str, relation_name: str) -> List[str]:
    """Reads list of streets for an area from OSM."""
    ret = get_nth_column(os.path.join(workdir, "streets-%s.csv" % relation_name), 1)
    ret += get_nth_column(os.path.join(workdir, "street-housenumbers-%s.csv" % relation_name), 1)
    return sorted(set(ret))


def get_workdir(config):
    """Gets the directory which is writable."""
    return config.get('wsgi', 'workdir').strip()


def process_template(buf, osmrelation):
    """Turns an overpass query template to an actual query."""
    buf = buf.replace("@RELATION@", str(osmrelation))
    # area is relation + 3600000000 (3600000000 == relation), see js/ide.js
    # in https://github.com/tyrasd/overpass-turbo
    buf = buf.replace("@AREA@", str(3600000000 + osmrelation))
    return buf


def get_content(workdir, path):
    """Gets the content of a file in workdir."""
    ret = ""
    with open(os.path.join(workdir, path)) as sock:
        ret = sock.read()
    return ret


def load_normalizers(datadir: str, relation_name: str) -> Tuple[Dict[str, Ranges], Dict[str, str]]:
    """Loads filters which allow silencing false positives. The return value is a tuple of the
    normalizers itself and an OSM name -> ref name dictionary."""
    filter_dict = {}  # type: Dict[str, Ranges]
    ref_streets = {}  # type: Dict[str, str]

    path = os.path.join(datadir, "housenumber-filters-%s.yaml" % relation_name)
    if not os.path.exists(path):
        return filter_dict, ref_streets

    with open(path) as sock:
        root = yaml.load(sock)

    if "filters" in root.keys():
        filters = root["filters"]
        for street in filters.keys():
            i = []
            if "ranges" not in filters[street]:
                continue
            for start_end in filters[street]["ranges"]:
                i.append(Range(int(start_end["start"]), int(start_end["end"])))
            filter_dict[street] = Ranges(i)

    if "refstreets" in root.keys():
        ref_streets = root["refstreets"]

    return filter_dict, ref_streets


def tsv_to_list(sock):
    """Turns a tab-separated table into a list of lists."""
    table = []

    for line in sock.readlines():
        if not line.strip():
            continue
        cells = line.split("\t")
        table.append(cells)

    return table


def html_table_from_list(table):
    """Produces a HTML table from a list of lists."""
    ret = []
    ret.append('<table rules="all" frame="border" cellpadding="4" class="sortable">')
    for row_index, row_content in enumerate(table):
        ret.append("<tr>")
        for cell in row_content:
            if row_index == 0:
                ret.append('<th align="left" valign="center"><a href="#">' + cell + "</a></th>")
            else:
                ret.append('<td align="left" valign="top">' + cell + "</td>")
        ret.append("</tr>")
    ret.append("</table>")
    return "".join(ret)


def normalize(house_numbers: str, street_name: str,
              normalizers: Dict[str, Ranges]) -> List[str]:
    """Strips down string input to bare minimum that can be interpreted as an
    actual number. Think about a/b, a-b, and so on."""
    ret = []
    for house_number in house_numbers.split('-'):
        try:
            number = int(re.sub(r"([0-9]+).*", r"\1", house_number))
        except ValueError:
            continue

        if street_name in normalizers.keys():
            # Have a custom filter.
            normalizer = normalizers[street_name]
        else:
            # Default sanity checks.
            default = [Range(1, 999), Range(2, 998)]
            normalizer = Ranges(default)
        if number not in normalizer:
            continue

        ret.append(str(number))
    return ret


def get_house_numbers_from_lst(workdir, relation_name, street_name, ref_street, normalizers):
    """Gets house numbers from reference."""
    house_numbers = []  # type: List[str]
    lst_street_name = ref_street
    prefix = lst_street_name + " "
    with open(os.path.join(workdir, "street-housenumbers-reference-%s.lst" % relation_name)) as sock:
        for line in sock.readlines():
            line = line.strip()
            if line.startswith(prefix):
                house_number = line.replace(prefix, '')
                house_numbers += normalize(house_number, street_name, normalizers)
    return sort_numerically(set(house_numbers))


def get_house_numbers_from_csv(workdir, relation_name, street_name, normalizers):
    """Gets house numbers from the overpass query."""
    house_numbers = []  # type: List[str]
    with open(os.path.join(workdir, "street-housenumbers-%s.csv" % relation_name)) as sock:
        first = True
        for line in sock.readlines():
            if first:
                first = False
                continue
            tokens = line.strip().split('\t')
            if len(tokens) < 3:
                continue
            if tokens[1] != street_name:
                continue
            house_numbers += normalize(tokens[2], street_name, normalizers)
    return sort_numerically(set(house_numbers))


def get_suspicious_streets(datadir, workdir, relation_name):
    """Tries to find streets which do have at least one house number, but are suspicious as other
    house numbers are probably missing."""
    suspicious_streets = []
    done_streets = []

    street_names = get_streets(workdir, relation_name)
    normalizers, ref_streets = load_normalizers(datadir, relation_name)
    for street_name in street_names:
        ref_street = street_name
        # See if we need to map the OSM name to ref name.
        if street_name in ref_streets.keys():
            ref_street = ref_streets[street_name]

        reference_house_numbers = get_house_numbers_from_lst(workdir, relation_name, street_name,
                                                             ref_street, normalizers)
        osm_house_numbers = get_house_numbers_from_csv(workdir, relation_name, street_name, normalizers)
        only_in_reference = get_only_in_first(reference_house_numbers, osm_house_numbers)
        in_both = get_in_both(reference_house_numbers, osm_house_numbers)
        if only_in_reference:
            suspicious_streets.append((street_name, only_in_reference))
        if in_both:
            done_streets.append((street_name, in_both))
    # Sort by length.
    suspicious_streets.sort(key=lambda result: len(result[1]), reverse=True)

    return suspicious_streets, done_streets


def build_reference_cache(local):
    """Builds an in-memory cache from the reference on-disk TSV."""
    memory_cache = {}  # type: Dict[str, Dict[str, Dict[str, List[str]]]]

    disk_cache = local + ".pickle"
    if os.path.exists(disk_cache):
        with open(disk_cache, "rb") as sock_cache:
            return pickle.load(sock_cache)

    with open(local, "r") as sock:
        first = True
        while True:
            line = sock.readline()
            if first:
                first = False
                continue

            if not line:
                break

            refmegye, reftelepules, street, num = line.strip().split("\t")
            if refmegye not in memory_cache.keys():
                memory_cache[refmegye] = {}
            if reftelepules not in memory_cache[refmegye].keys():
                memory_cache[refmegye][reftelepules] = {}
            if street not in memory_cache[refmegye][reftelepules].keys():
                memory_cache[refmegye][reftelepules][street] = []
            memory_cache[refmegye][reftelepules][street].append(num)
    with open(disk_cache, "wb") as sock_cache:
        pickle.dump(memory_cache, sock_cache)
    return memory_cache


def house_numbers_of_street(datadir, reference, relation_name, street):
    """Gets house numbers for a street locally."""
    refmegye, reftelepules_list, street_name, street_type = get_street_details(datadir, street, relation_name)
    street = street_name + " " + street_type
    ret = []  # type: List[str]
    for reftelepules in reftelepules_list:
        if street in reference[refmegye][reftelepules].keys():
            house_numbers = reference[refmegye][reftelepules][street]
            ret += [street + " " + i for i in house_numbers]

    return ret


def get_reference_housenumbers(reference, datadir, workdir, relation_name):
    """Gets known house numbers (not their coordinates) from a reference site, based on street names
    from OSM."""
    memory_cache = build_reference_cache(reference)

    streets = get_streets(workdir, relation_name)

    lst = []  # type: List[str]
    for street in streets:
        lst += house_numbers_of_street(datadir, memory_cache, relation_name, street)

    lst = sorted(set(lst))
    sock = open(os.path.join(workdir, "street-housenumbers-reference-%s.lst" % relation_name), "w")
    for line in lst:
        sock.write(line + "\n")
    sock.close()


def get_relations(datadir):
    """Returns a name -> properties dictionary."""
    with open(os.path.join(datadir, "relations.yaml")) as sock:
        return yaml.load(sock)


def get_streets_query(datadir, relations, relation):
    """Produces a query which lists streets in relation."""
    with open(os.path.join(datadir, "streets-template.txt")) as sock:
        return process_template(sock.read(), relations[relation]["osmrelation"])

# vim:set shiftwidth=4 softtabstop=4 expandtab:
