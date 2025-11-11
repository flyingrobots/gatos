#!/usr/bin/env node
// Simple invariant check: envelope.ns MUST equal the CLI/topic segment
// Usage: check_topic_ns.js <path-to-envelope.json> <topic>
const fs = require('fs');

function usage() {
  console.error('usage: check_topic_ns.js <envelope.json> <topic>');
  process.exit(2);
}

const [, , file, topic] = process.argv;
if (!file || !topic) usage();

let ns;
try {
  const text = fs.readFileSync(file, 'utf8');
  const obj = JSON.parse(text);
  ns = obj.ns;
} catch (e) {
  console.error('failed to read/parse envelope:', e.message);
  process.exit(2);
}

if (ns !== topic) {
  console.error(`topic/ns mismatch: topic=${topic} ns=${ns}`);
  process.exit(1);
}
// match → success (no output)
process.exit(0);

