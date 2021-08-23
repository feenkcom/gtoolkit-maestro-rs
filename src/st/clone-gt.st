EpMonitor current disable.

Metacello new
    baseline: 'GToolkitReleaser';
    repository: 'github://feenkcom/gtoolkit-releaser:main/src';
    load.

Metacello new
    baseline: 'Beacon';
    repository: 'github://feenkcom/pharo-beacon/src';
    onConflictUseIncoming;
    load.

MCCacheRepository disableCacheDuring: [
    [ #GtRlProjectsLoader asClass new
        loadBaseline: 'GToolkitForPharo9'
        fromRepository: 'github://feenkcom/gtoolkit:main/src' ]
            on: Warning
            do: [ :anException |
                MetacelloNotification signal: anException description.
            	anException resume ] ].

EpMonitor current enable.

PharoLightTheme beCurrent.
World closeAllWindowsDiscardingChanges.
