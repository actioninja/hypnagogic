# Cutter2
### A multipurpose icon processing tool for byond

## Configuration

Cutter2 needs configuration to operate on! 

Configuration is as simple as creating a .yaml file with the same name

See `examples` for deeper documentation on the config format, as well as `in_test` for some
simpler examples.

Some basic templates are offered in `templates` for various common scenarios.

## Usage

Basic usage is as simple as

`cutter2 input_dir`

This will deep search the directory for .yaml/yml files and attempt to perform an operation
on files with matching names.

Cutter2 offers a command line help tool! See it for possible command line flags

`cutter2 -help`
