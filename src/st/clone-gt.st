EpMonitor current disable.

Metacello new
    baseline: 'GToolkitReleaser';
    repository: 'github://feenkcom/gtoolkit-releaser:main/src';
    load.

GtRlProjectsLoader new
    loadBaseline: 'GToolkitForPharo9'
    fromRepository: 'github://feenkcom/gtoolkit:main/src'.

EpMonitor current enable.

PharoLightTheme beCurrent.
World closeAllWindowsDiscardingChanges.