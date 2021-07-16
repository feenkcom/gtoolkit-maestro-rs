EpMonitor current disable.

Metacello new
    baseline: 'GToolkitReleaser';
    repository: 'github://feenkcom/gtoolkit-releaser/src';
    load.

GtRlProjectsLoader new
    loadBaseline: 'GToolkitForPharo9'
    fromRepository: 'github://feenkcom/gtoolkit/src'.

EpMonitor current enable.

PharoLightTheme beCurrent.
World closeAllWindowsDiscardingChanges.