#!/bin/bash

# Move to script's directory
cd `dirname "$0"`

# if it's a symlink, move to the real directory
if [ -L "$0" ]; then
    cd `dirname $(readlink "$0")`
fi

# Get the kernel/architecture information
ARCH=`uname -m`

# Set default ld lib path if not set
if [ -z "$LD_LIBRARY_PATH" ]; then
    export LD_LIBRARY_PATH=/usr/lib
fi

# Set the libpath and pick the proper binary
if [ "$ARCH" == "x86_64" ]; then
    export LD_LIBRARY_PATH=$LD_LIBRARY_PATH:"`dirname "."`"/x86_64/
    echo $LD_LIBRARY_PATH
    ./ppan.x86_64 $@
else
    export LD_LIBRARY_PATH=$LD_LIBRARY_PATH:"`dirname "."`"/x86/
    ./ppan.x86 $@
fi
