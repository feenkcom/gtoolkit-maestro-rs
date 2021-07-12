#!/bin/bash

# gtoolkit.sh
# Downloads the gtoolkit-maestro app for the current platform which orchestrates the local build of the Glamorous Toolkit
# Consult https://github.com/feenkcom/gtoolkit-maestro-rs for more information.
# When no arguments are passed to the gtoolkit.sh script the default "build" argument will be used instead

arguments=$*
if [ $# -eq 0 ]
  then
    arguments="build"
fi

if [[ "$OSTYPE" == "linux-gnu"* ]]; then
  if [ ! -f "gtoolkit" ]; then
    curl -o gtoolkit -L -s -S 'https://github.com/feenkcom/gtoolkit-maestro-rs/releases/latest/download/gtoolkit-x86_64-unknown-linux-gnu' > /dev/null
  fi
  chmod +x gtoolkit
  ./gtoolkit $arguments
elif [[ "$OSTYPE" == "darwin"* ]]; then

  if [ ! -f "gtoolkit" ]; then
    arch_name="$(uname -m)"
    is_m1=false
    if [ "${arch_name}" = "x86_64" ]; then
        if [ "$(sysctl -in sysctl.proc_translated)" = "1" ]; then
            is_m1=true
        fi
    elif [ "${arch_name}" = "arm64" ]; then
        is_m1=true
    fi

    if [[ "$is_m1" == true ]]; then
      curl -o gtoolkit -L -s -S 'https://github.com/feenkcom/gtoolkit-maestro-rs/releases/latest/download/gtoolkit-aarch64-apple-darwin' > /dev/null
    else
      curl -o gtoolkit -L -s -S 'https://github.com/feenkcom/gtoolkit-maestro-rs/releases/latest/download/gtoolkit-x86_64-apple-darwin' > /dev/null
    fi
  fi
  chmod +x gtoolkit
  ./gtoolkit $arguments

elif [[ "$OSTYPE" == "cygwin" || "$OSTYPE" == "msys" || "$OSTYPE" == "win32" ]]; then
  if [ ! -f "gtoolkit.exe" ]; then
    curl -o gtoolkit.exe -L -s -S 'https://github.com/feenkcom/gtoolkit-maestro-rs/releases/latest/download/gtoolkit-x86_64-pc-windows-msvc.exe' > /dev/null
  fi
  chmod +x gtoolkit.exe
  ./gtoolkit.exe $arguments
else
  echo "$OSTYPE is unsupported. Please submit an issue at https://github.com/feenkcom/gtoolkit/issues".
  exit 1
fi

exit 0