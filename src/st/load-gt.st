Iceberg remoteTypeSelector: #httpsUrl.
EpMonitor current disable.

Metacello new
    baseline: 'GToolkitForPharo9';
    repository: 'github://feenkcom/gtoolkit/src';
    silently;
    onConflictUseIncoming;
    load.

EpMonitor current enable.

PharoLightTheme beCurrent.
#GtImageSetup asClass performLocalSetup.
World closeAllWindowsDiscardingChanges.
Smalltalk snapshot: true andQuit: true.
