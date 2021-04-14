# PySprint-CLI

[![Build status](https://ci.appveyor.com/api/projects/status/tmnlqvcsoumeq591?svg=true)](https://ci.appveyor.com/project/Ptrskay3/pysprint-cli)

[![Build Status](https://travis-ci.com/Ptrskay3/pysprint-cli.svg?branch=master)](https://travis-ci.com/Ptrskay3/pysprint-cli)

![CI](https://github.com/Ptrskay3/pysprint-cli/actions/workflows/ci.yml/badge.svg)

The command line tool for PySprint to boost productivity.

### WARNING!

PySprint-CLI is in very early stages of development, you might find bugs or undefined behaviour.

### Usage

First, set up an `eval.yaml` file where you will work. PySprint-CLI will optionally generate a default one on demand. That file will define the behaviour of the program. The full key and value list will be included soon. Here is an example:

```yml
load_options:
  - skiprows: 8 # lines
  - decimal: ","
  - delimiter: ";"
  - meta_len: 6 # lines

preprocess:
  - input_unit: "nm"
  - chdomain: true
  - slice_start: 2 # PHz
  - slice_stop: 4 # PHz

method:
  - wft

method_details:
  - heatmap
  - windows: 200 # number of windows
  - fwhm: 0.05 # PHz

before_evaluate:
  - "print('this is a point where you can')"
  - "print('interact with the program')"

evaluate:
  - reference_frequency: 2.355 # PHz
  - order: 3 # up to TOD

after_evaluate:
  - "print('and also here, after evaluate..')"
```

To start watching a directory, run:

```shell
pysprint-cli watch [your/path/here]
```

To run an evaluation on an already existing filebase, run:

```shell
pysprint-cli audit [your/path/here]
```

Optionally generated files can be saved with the `--persist` (or `-p` for short) flag.

### Building from source

Make sure you have [Rust](https://www.rust-lang.org/tools/install) and [Python 3.6+](https://www.python.org/downloads/) installed.
You will also need [Build Tools for Visual Studio](https://visualstudio.microsoft.com/downloads/#build-tools-for-visual-studio-2019).

To install locally, navigate to the root directory of the crate, and run:

```shell
cargo install --path .
```
