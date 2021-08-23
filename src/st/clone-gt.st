EpMonitor current disable.

Metacello new
    baseline: 'GToolkitReleaser';
    repository: 'github://feenkcom/gtoolkit-releaser:main/src';
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
