| patcher |

EpMonitor current disable.
Iceberg enableMetacelloIntegration: false.

Metacello new
    githubUser: 'syrel' project: 'pharo9-backport' commitish: 'main' path: '';
    baseline: 'PharoPatcher';
    load.

patcher := #PharoPatcher asClass.

NonInteractiveTranscript stdout
    nextPutAll: 'Patching Pharo9 image...';
    cr.

patcher patch: Context source:
'copyTo: aContext
    "Copy self and my sender chain down to, but not including, aContext.
    End of copied chain will have nil sender."
    | copy |
    self == aContext ifTrue: [^ nil].
    "Workaround for the regular VM crash that occurs when copying self below.
    Accessing the pc seems to be enough to prevent the crash."
    self pc.
    copy := self copy.
    self sender ifNotNil: [
        copy privSender: (self sender copyTo: aContext)].
    ^ copy'.


patcher patch: Object source:
'setPinnedInMemory: aBoolean
 	"The VM''s garbage collector routinely moves objects as it reclaims and compacts
 	 memory. But it can also pin an object so that it will not be moved around in memory,
     while still being reclamable by the garbage collector. This can make
 	 it easier to pass objects out through the FFI. Objects are unpinnned when created.
 	 This primitive either pins or unpins an object, and answers if it was already pinned.

 	If there is not enough memory, I will try to find more memory and retry once."
 	<primitive: 184 error: ec>

 	ec = #''insufficient object memory''
 		ifFalse: [ ^ self primitiveFailed ].

 	Smalltalk garbageCollect < (self sizeInMemory * 2) ifTrue:
 		[Smalltalk growMemoryByAtLeast: self sizeInMemory * 2 ].

 	^ self retrySetPinnedInMemory: aBoolean'.

patcher patch: Metaclass source:
'hasClassVarNamed: aString
 	"Return whether the receiver has a class variables (shared variables among its class and subclasses) named: aString"

 	^self instanceSide
 		ifNil: [ false ]
 		ifNotNil: [ :class | class hasClassVarNamed: aString ]'.

patcher patch: Metaclass source:
'classVariableNamed: aString ifAbsent: absentBlock
	"Answer the Class Variable"

	^ self instanceSide
		ifNil: absentBlock
		ifNotNil: [ :class | class classVariableNamed: aString ifAbsent: absentBlock ]'.

patcher patch: Month class source:
'indexOfMonth: aMonthName
	1 to: 12 do: [ :i |
		((MonthNames at: i) beginsWithEmpty:  aMonthName caseSensitive: false) ifTrue: [^i] ].
 	self error: aMonthName , '' is not a recognized month name'''.

patcher patch: RPackageOrganizer source: 'packageMatchingExtensionName: anExtensionName
	"return nil if no package is not found"
	| tmpPackageName |

	"we first look if their is a package matching exactly the name specified"
	(self packageExactlyMatchingExtensionName: anExtensionName)
		ifNotNil: [ :package | ^ package ].

	"if no package was found, we try to find one matching the begining of the name specified"
	tmpPackageName := ''''.
	packages keysDo: [:aSymbol |
		(anExtensionName beginsWithEmpty: (aSymbol asString, ''-'') caseSensitive: false)
			ifTrue: [
				"we keep the longest package name found"
				(aSymbol size > tmpPackageName size)
					ifTrue: [ tmpPackageName := aSymbol ]]].

	^ tmpPackageName = ''''
		ifTrue: [ nil ]
		ifFalse: [ self packageNamed: tmpPackageName ]'.

patcher patch: RPackageOrganizer source: 'hasPackageMatchingExtensionName: anExtensionName
	(self hasPackageExactlyMatchingExtensionName: anExtensionName)
		ifTrue: [ ^true ].

	packages keysDo: [ :aSymbol |
		(anExtensionName beginsWithEmpty: (aSymbol asString, ''-'') caseSensitive: false)
			ifTrue: [ ^ true]].
	^ false'.

patcher patch: MCPackageLoader source: 'tryToLoad: aDefinition
	aDefinition isClassDefinition ifTrue: [ ^ false ].

	[aDefinition addMethodAdditionTo: methodAdditions] on: Error do: [errorDefinitions add: aDefinition].'.

patcher patch: CompiledMethod source: 'basicAsMCMethodDefinition
	^ MCMethodDefinition
		  className: self methodClass instanceSide name
		  classIsMeta: self methodClass isClassSide
		  selector: self selector
		  category: self protocol
		  timeStamp: '''' "self stamp"
		  source: self sourceCode'.

patcher patch: Object source: 'gtDisplayString
    "This offers a means to customize how the object is shown in the GlamorousToolkit UI.
    Do not override this method. Override gtDisplayOn: instead"

    | limitedString limit |
    limit := 1000.
    limitedString := String streamContents: [:s | self gtDisplayOn: s] limitedTo: limit.
    limitedString size < limit ifTrue: [^ limitedString].
    ^ limitedString , '' ...'''
classified: 'printing'.

patcher patch: Object source: 'gtDisplayOn: stream
	"This offers a means to customize how the object is shown in the GlamorousToolkit UI"
	^ self printOn: stream'
classified: 'printing'.

patcher patch: ExternalData source: 'readStringUTF8
	"Assume that the receiver represents a C string containing UTF8 characters and convert
	 it to a Smalltalk string."
	| stream index char |

	self isNull ifTrue: [ ^ nil ].

	type isPointerType ifFalse: [self error: ''External object is not a pointer type.''].
	stream := WriteStream on: ByteArray new.
	index := 1.
	[(char := handle unsignedByteAt: index) = 0 ] whileFalse: [
		stream nextPut: char.
		index := index + 1].
	^ [ ZnCharacterEncoder utf8 decodeBytes: stream contents ] on: ZnInvalidUTF8 do: [ stream contents asString ]'
classified: '*UnifiedFFI'.

patcher patch: CommandLineUIManager source: 'activate
	NonInteractiveTranscript stdout install'
classified: 'private'.

patcher patch: #LGitLibrary asClass source:
'macLibraryName

 	^ FFIMacLibraryFinder findAnyLibrary: #(''libgit2.dylib'' ''libgit2.1.0.1.dylib'' ''libgit2.1.0.0.dylib'' ''libgit2.0.25.1.dylib'')'
classified: 'accessing - platform'.

patcher patch: #LGitLibrary asClass source:
'unix64LibraryName

	^ FFIUnix64LibraryFinder findAnyLibrary: #(
	    ''libgit2.so''
		"This name is wrong, but some versions of the VM has this library shipped with the bad name"
		''libgit2.1.0.0.so''
		''libgit2.so.1.0.0''
		''libgit2.so.1.0''
		''libgit2.so.1.1''
		''libgit2.so.0.25.1'')'
classified: 'accessing - platform'.

patcher patch: #LGitLibrary asClass source:
'win32LibraryName

    ^ FFIWindowsLibraryFinder findAnyLibrary: #(''git2.dll'' ''libgit2-1-0-0.dll'' ''libgit2.dll'')'
classified: 'accessing - platform'.

patcher patch: #FFIUnixLibraryFinder asClass source:
'basePaths
	^ {
	(Smalltalk vm directory asFileReference parent / ''lib'') pathString.
	Smalltalk imageDirectory fullName.
	Smalltalk vm directory }'
classified: 'accessing'.

patcher patch: #FFIUnix64LibraryFinder asClass source:
'knownPaths
	^ #(
    	''/lib/x86_64-linux-gnu''
    	''/lib64''
    	''/usr/lib64''
    	''/usr/lib''
    	''/usr/lib/x86_64-linux-gnu''
    	''/usr/local/lib'')'
classified: 'accessing'.

patcher patch: #CairoLibrary asClass source:
'macLibraryName

 	^ FFIMacLibraryFinder findAnyLibrary: #(''libcairo.dylib'' ''libcairo.2.dylib'')'
classified: 'accessing - platform'.
patcher patch: #CairoLibrary asClass source:
'win32LibraryName

 	^ FFIWindowsLibraryFinder findAnyLibrary: #(''cairo.dll'' ''libcairo-2.dll'')'
classified: 'accessing - platform'.

ZnHTTPSTest removeSelector: #testGForceInria.
NonInteractiveTranscript stdout
	nextPutAll: '   ZnHTTPSTest>>#testGForceInria removed.'; cr.

EpMonitor current enable.
Iceberg enableMetacelloIntegration: true.
