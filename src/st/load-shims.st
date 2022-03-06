EpMonitor current disable.
Iceberg enableMetacelloIntegration: false.

Metacello new
    githubUser: 'syrel' project: 'pharo-shims' commitish: 'main' path: 'src';
    baseline: 'PharoShims';
    onConflictUseIncoming;
    load.
NonInteractiveTranscript stdout
    nextPutAll: 'Loaded PharoShims';
    cr.
Metacello new
    githubUser: 'pharo-contributions' project: 'CollectionExtensions' commitish: 'v1.x.x' path: 'src';
    baseline: 'CollectionExtensions';
    load.
NonInteractiveTranscript stdout
    nextPutAll: 'CollectionExtensions';
    cr.

Iceberg enableMetacelloIntegration: true.
EpMonitor current enable