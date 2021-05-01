# PySprint-CLI

[![Build status](https://ci.appveyor.com/api/projects/status/tmnlqvcsoumeq591?svg=true)](https://ci.appveyor.com/project/Ptrskay3/pysprint-cli)

[![Build Status](https://travis-ci.com/Ptrskay3/pysprint-cli.svg?branch=master)](https://travis-ci.com/Ptrskay3/pysprint-cli)

![CI](https://github.com/Ptrskay3/pysprint-cli/actions/workflows/ci.yml/badge.svg)

The command line tool for PySprint to boost productivity.

### WARNING!

PySprint-CLI is in very early stages of development, you might find bugs or undefined behaviour.

### Usage

First, set up an `eval.yaml` file where you will work. PySprint-CLI will optionally generate a default one on demand. That file will define the behaviour of the program. Here is an example:

```yml
load_options:
  extensions:
    - "trt"
    - "txt"
  exclude_patterns:
    - "*?3?.trt"
  skip_files:
    - "my_file_to_skip.txt"
  skiprows: 8
  decimal: ","
  delimiter: ";"
  meta_len: 6
  mod: -1
  no_comment_check: true
preprocess:
  chdomain: true
  input_unit: "nm"
  slice_start: 2
  slice_stop: 4
method: fft
method_details:
  heatmap: false
  windows: 200
  fwhm: 0.05
  std: 0.05
  parallel: false
  plot: false
  only_phase: false
  min: false
  max: false
  both: false
  eager: false
  detach: true
before_evaluate:
  - print('before_evaluate')
  - print('you have access to the `ifg` variable')
evaluate:
  reference_frequency: 2.355
  order: 3
after_evaluate:
  - "print('and after evaluate too..')"
```

To start watching a directory, run:

```shell
psc watch [your/path/here]
```

To run an evaluation on an already existing filebase, run:

```shell
psc audit [your/path/here]
```

Optionally generated files can be saved with the `--persist` flag.

### Building from source

Make sure you have [Rust](https://www.rust-lang.org/tools/install) and [Python 3.6+](https://www.python.org/downloads/) installed.
You will also need [Build Tools for Visual Studio](https://visualstudio.microsoft.com/downloads/#build-tools-for-visual-studio-2019).

To install locally, navigate to the root directory of the crate, and run:

```shell
cargo install --path .
```
