#!/bin/sh

# AAAH
echo -nE id$'\x1f'`pwd`$'\x1e' | ./gitstatusd/usrbin/gitstatusd | bat -A | sed 's/\\u{1f}/#/g' | tr \# '\n'