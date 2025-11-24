#!/usr/bin/env bash

if [ -z "$1" ]; then
  echo "provide the package name to run as the first argument"
  exit 1
fi

exe="$1/out/a.out"
shift
$exe $@
