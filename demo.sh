#!/usr/bin/env bash
set -e

# Run demo with hot reload
exec cargo watch -x 'run -- --demo --fast'
