#!/bin/bash

# installer.sh
# Downloads the gtoolkit-maestro app for the current platform which orchestrates the local build of the Glamorous Toolkit
# Consult https://github.com/feenkcom/gtoolkit-maestro-rs for more information.
# When no arguments are passed to the gtoolkit.sh script the default "build" argument will be used instead

installer="gt-installer"

arguments=$*
if [ $# -eq 0 ]
  then
    arguments="build"
fi

if [[ "$OSTYPE" == "linux-gnu"* ]]; then
  if [ ! -f "$installer" ]; then
    curl -o "$installer" -L -s -S "https://github.com/feenkcom/gtoolkit-maestro-rs/releases/latest/download/$installer-x86_64-unknown-linux-gnu" > /dev/null
  fi
  chmod +x "$installer"
  ./$installer $arguments
elif [[ "$OSTYPE" == "darwin"* ]]; then

  if [ ! -f "$installer" ]; then
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
      curl -o "$installer" -L -s -S "https://github.com/feenkcom/gtoolkit-maestro-rs/releases/latest/download/$installer-aarch64-apple-darwin" > /dev/null
    else
      curl -o "$installer" -L -s -S "https://github.com/feenkcom/gtoolkit-maestro-rs/releases/latest/download/$installer-x86_64-apple-darwin" > /dev/null
    fi
  fi
  chmod +x "$installer"
  ./$installer $arguments

elif [[ "$OSTYPE" == "cygwin" || "$OSTYPE" == "msys" || "$OSTYPE" == "win32" ]]; then
  if [ ! -f "$installer.exe" ]; then
    curl -o "$installer.exe" -L -s -S "https://github.com/feenkcom/gtoolkit-maestro-rs/releases/latest/download/$installer-x86_64-pc-windows-msvc.exe" > /dev/null
  fi
  chmod +x "$installer.exe"
  ./$installer.exe $arguments
else
  echo "$OSTYPE is unsupported. Please submit an issue at https://github.com/feenkcom/gtoolkit/issues".
  exit 1
fi

exit 0