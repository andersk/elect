#!/usr/bin/env python
# -*- coding: utf-8 -*-

# Copyright © 2016 Anders Kaseorg <andersk@mit.edu>
#
# This program is free software: you can redistribute it and/or modify
# it under the terms of the GNU General Public License as published by
# the Free Software Foundation, either version 3 of the License, or
# (at your option) any later version.
#
# This program is distributed in the hope that it will be useful, but
# WITHOUT ANY WARRANTY; without even the implied warranty of
# MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU
# General Public License for more details.
#
# You should have received a copy of the GNU General Public License
# along with this program.  If not, see
# <http://www.gnu.org/licenses/>.

# Requires the python-vote-core library
# <https://github.com/bradbeattie/python-vote-core>:
#
#   pip install python-vote-core

from __future__ import print_function

import fileinput
import itertools
import operator
import optparse
import sys

from pyvotecore.schulze_stv import SchulzeSTV

try:
    BALLOT_NOTATION_GROUPING = SchulzeSTV.BALLOT_NOTATION_GROUPING
except AttributeError:
    BALLOT_NOTATION_GROUPING = 'grouping'

usage = '''\
usage: %prog [-w N|--winners=N] [-b|--break-ties] BALLOTFILE...

Each BALLOTFILE has one ballot description per line, with candidate
names separated by > or = to indicate strict and equal preference.
Prefixing a ballot with WEIGHT: makes WEIGHT copies of it.

  Chocolate > Vanilla > Strawberry > Cookie Dough
  Cookie Dough > Chocolate > Strawberry
  2: Strawberry = Chocolate > Vanilla

Candidate names are case-sensitive, and may include whitespace but may
not include > or =.  Whitespace around operators is ignored.
Candidates not listed in a ballot will be treated as tied for least
preferred.

Pass - to read ballots from stdin.'''

parser = optparse.OptionParser(usage=usage)
parser.add_option('-w', '--winners', type='int', dest='winners', default=1,
                  help='elect an N-winner committee (default: 1)', metavar='N')
parser.add_option('-b', '--break-ties', action='store_true', dest='break_ties',
                  help='break ties randomly')

def parse_ballot(line):
    count = 1
    if ':' in line:
        count, line = line.split(':', 1)
        count = int(count)
    return {
        'count': count,
        'ballot': [[candidate.strip() for candidate in group.split('=')]
                   for group in line.split('>')],
    }

def parse_ballots(args):
    return [parse_ballot(line) for line in fileinput.input(args) if line.strip()]

def show_result(opts, output):
    print('Candidates ({}):'.format(len(output.candidates)))
    for candidate in sorted(output.candidates):
        print('  {}'.format(candidate))
    print()

    print('Ballots ({}):'.format(sum(ballot['count'] for ballot in output.ballots)))
    for ballot in output.ballots:
        groups = itertools.groupby(
            sorted((-rating, candidate) for candidate, rating in
                   ballot['ballot'].items()),
            key=operator.itemgetter(0))
        print('  {}: {}'.format(
            ballot['count'],
            ' > '.join(
                ' = '.join(candidate for _, candidate in group)
                for _, group in groups)))
    print()

    set_suffix = ' set' if output.required_winners != 1 else ''

    if hasattr(output, 'tied_winners'):
        print('Tied winner{}s:'.format(set_suffix))
        for winners in output.tied_winners:
            print('  {}'.format(', '.join(sorted(winners))))
        if opts.break_ties:
            print('warning: Breaking tie randomly.')
            print()
        else:
            print('Pass -b (--break-ties) to break ties randomly.')

    if not hasattr(output, 'tied_winners') or opts.break_ties:
        print('Winner{}:'.format(set_suffix))
        print('  {}'.format(', '.join(sorted(output.winners))))

def main():
    opts, args = parser.parse_args()
    if not args:
        parser.print_usage(file=sys.stderr)
        return 1

    ballots = parse_ballots(args)
    if not ballots:
        print('error: No ballots found', file=sys.stderr)
        return 1

    print('Tallying Schulze STV election.')
    print()
    output = SchulzeSTV(
        ballots,
        required_winners=opts.winners,
        ballot_notation=BALLOT_NOTATION_GROUPING,
    )
    show_result(opts, output)

if __name__ == '__main__':
    sys.exit(main())
