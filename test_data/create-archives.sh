#!/bin/bash

# exit if any of the archive/compression commands fails
set -e

rm -f root.{zip,tar{,.gz,.xz,.bz2}}
#rm -f root.zip root.tar root.tar.gz root.tar.xz root.tar.

pushd root >/dev/null
zip -r ../root.zip *
tar -cvf ../root.tar *
popd >/dev/null

gzip -k root.tar
xz -k root.tar
bzip2 -k root.tar
