#!/usr/bin/env bash
set -euo pipefail

# Fail if any line contains 'Schema:' that is not 'Envelope-Schema:' or 'Trailer-Schema:'
if rg -n "\\bSchema:\\b" docs | rg -v "Envelope-Schema|Trailer-Schema" -n | sed -n '1,200p'; then
  echo "Found legacy 'Schema:' header(s). Use Envelope-Schema and Trailer-Schema only." >&2
  exit 1
fi
echo "ok: no legacy 'Schema:' headers"

