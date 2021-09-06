EpMonitor current disable.

Metacello new
    baseline: 'GToolkitForPharo9';
    repository: 'github://feenkcom/gtoolkit:{{gtoolkit_version}}/src';
    silently;
    onConflictUseIncoming;
    load.

EpMonitor current enable.

PharoLightTheme beCurrent.
World closeAllWindowsDiscardingChanges.