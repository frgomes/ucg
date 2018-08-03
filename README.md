# Universal Configuration Grammar.

[![Build Status](https://travis-ci.org/zaphar/ucg.svg?branch=master)](https://travis-ci.org/zaphar/ucg)

This is an experiment in configuration management. The approach is **not**
to create a "parsable" config file format.  We have plenty of
those. Instead we try to specify a grammar for describing
configuration values that can then target various configuration
formats to output to.

In theory this could support anything from command line flags to json
to yaml or toml or even xml.

The goal is to allow a global shared configuration repository that can
be version controlled, enforce _some_ typesafety, and output
configuration for any application regardless of that applications
preferred format.

## The Usual Warnings

This is still very much an experiment and the language and api can be expected
to change and mutate. It also probably has bugs and isn't yet the most user
friendly language and compiler to use. You have been warned.

## Usage

You can get ucg with cargo `cargo install ucg`.

Running ucg help will show the following output.

```sh
Universal Configuration Grammar compiler.

USAGE:
    ucg [SUBCOMMAND]

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

SUBCOMMANDS:
    build       Compile a specific ucg file.
    help        Prints this message or the help of the given subcommand(s)
    validate    Check a specific ucg file for errors.
```

## compiling a file

```sh
Compile a specific ucg file.

USAGE:
    ucg build [OPTIONS] <INPUT> --sym <sym> --target <target>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -o, --out <out>          Output file to write to.
        --sym <sym>          Specify a specific let binding in the ucg file to output.
    -t, --target <target>    Target output type. (flags, json, env, exec)

ARGS:
    <INPUT>    Input ucg file to build.
```

## Language Reference

https://docs.rs/ucg/