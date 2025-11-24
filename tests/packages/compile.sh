#!/usr/bin/env bash

if [ -z "$1" ]; then
  echo "provide at least one package name to compile"
  exit 1
fi

objs=()
for package in "$@"; do
  echo "Compiling $package..."
  clang "$package/out/$package.ll" -c -o "$package/out/$package.o"
  objs+=("$package/out/$package.o")
done
echo "Linking objects..."
clang "${objs[@]}" -o "$1/out/a.out"
echo "Finished."
