#!/usr/bin/env bash

set -e

# Find out where the pretty printer Python module is
RUSTC_SYSROOT=`rustc --print=sysroot`
GDB_PYTHON_MODULE_DIRECTORY="$RUSTC_SYSROOT/lib/rustlib/etc"

# Run GDB with the additional arguments that load the pretty printers
# Set the environment variable `RUST_GDB` to overwrite the call to a
# different/specific command (defaults to `gdb`).
PYTHONPATH="$PYTHONPATH:$GDB_PYTHON_MODULE_DIRECTORY" gdbgui -g ./rust-os-gdb/bin/gdb \
  --args "-d \"$GDB_PYTHON_MODULE_DIRECTORY\" -iex \"add-auto-load-safe-path $GDB_PYTHON_MODULE_DIRECTORY\" -ex \"target remote :1234\"" \
  -n
"$@"
