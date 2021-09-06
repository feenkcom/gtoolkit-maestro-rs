EpMonitor current disable.

Metacello new
    baseline: 'GToolkitReleaser';
    repository: 'github://feenkcom/gtoolkit-releaser:{{releaser_version}}/src';
    load.

MCCacheRepository disableCacheDuring: [
    [ #GtRlProjectsLoader asClass new
        loadBaseline: 'GToolkitForPharo9'
        fromRepository: 'github://feenkcom/gtoolkit:{{gtoolkit_version}}/src' ]
            on: Warning
            do: [ :anException |
                MetacelloNotification signal: anException description.
            	anException resume ] ].

EpMonitor current enable.

PharoLightTheme beCurrent.
World closeAllWindowsDiscardingChanges.
