Iceberg remoteTypeSelector: #httpsUrl.
EpMonitor current disable.

Metacello new
    baseline: 'GToolkitReleaser';
    githubUser: 'feenkcom' project: 'gtoolkit-releaser' commitish: 'master' path: '';
    load.

GtRlProjectsLoader new
    loadBaseline: 'GToolkit'
    fromRepository: 'github://feenkcom/gtoolkit/src'.

EpMonitor current enable.

PharoLightTheme beCurrent.
#GtImageSetup asClass performLocalSetup.
World closeAllWindowsDiscardingChanges.
Smalltalk snapshot: true andQuit: true.
