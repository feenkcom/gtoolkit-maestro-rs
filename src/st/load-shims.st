EpMonitor current disable.
Iceberg enableMetacelloIntegration: false.

Metacello new
    githubUser: 'pharo-contributions' project: 'CollectionExtensions' commitish: 'v1.x.x' path: 'src';
    baseline: 'CollectionExtensions';
    load.
NonInteractiveTranscript stdout
    nextPutAll: 'CollectionExtensions';
    cr.

Iceberg enableMetacelloIntegration: true.
EpMonitor current enable