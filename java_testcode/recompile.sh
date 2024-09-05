#!/usr/bin/env bash

#Poor mans make file for java. I only need to compile a bunch of loose test files, no point in using maven/gradle/ant for this.

sources=$(find . -type f -name "*.java")
classes=$(find . -type f -name "*.class")

for clz in "${classes[@]}"
do
  echo $clz
  rm $clz
done

for src in "${sources[@]}"
do
  echo $src
  javac --source 8 --target 8 $src
done