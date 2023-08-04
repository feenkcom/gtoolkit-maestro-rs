| externalAddressMethodsToPatch |

EpMonitor current disable.
Iceberg enableMetacelloIntegration: false.

NonInteractiveTranscript stdout
    nextPutAll: 'Patching Pharo 10 image...';
    cr.

externalAddressMethodsToPatch := #(
    boolean8AtOffset: boolean8AtOffset:put:
    char8AtOffset: char8AtOffset:put:
    char16AtOffset: char16AtOffset:put:
    char32AtOffset: char32AtOffset:put:
    int8AtOffset: int8AtOffset:put:
    int16AtOffset: int16AtOffset:put:
    int32AtOffset: int32AtOffset:put:
    int64AtOffset: int64AtOffset:put:
    uint8AtOffset: uint8AtOffset:put:
    uint16AtOffset: uint16AtOffset:put:
    uint32AtOffset: uint32AtOffset:put:
    uint64AtOffset: uint64AtOffset:put:).

externalAddressMethodsToPatch do: [ :each |
	(RBRemovePragmaTransformation
		pragma: '<primitive: nil>'
		inMethod: each
		inClass: #ExternalAddress) asRefactoring execute ].

Object compile:
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

 	^ self retrySetPinnedInMemory: aBoolean'
classified: (Object >> #setPinnedInMemory:) protocol.

Object compile:
'asOrderedCollection
	^ OrderedCollection with: self'
classified: (Object >> #asOrderedCollection) protocol.

Metaclass compile:
'classVariableNamed: aString ifAbsent: absentBlock
	"Answer the Class Variable"

	^ self instanceSide
		ifNil: absentBlock
		ifNotNil: [ :class | class classVariableNamed: aString ifAbsent: absentBlock ]'
classified: 'class variables'.

Month class compile:
'indexOfMonth: aMonthName
	1 to: 12 do: [ :i |
		((MonthNames at: i) beginsWith: aMonthName caseSensitive: false) ifTrue: [^i] ].
 	self error: aMonthName , '' is not a recognized month name'''
classified: (Month class >> #indexOfMonth:) protocol.

RPackage compile: 'toTagName: aSymbol
	^ (aSymbol beginsWith: self name asString, ''-'')
		ifTrue: [ (aSymbol allButFirst: self name size + 1) asSymbol ]
		ifFalse: [ aSymbol ]'
classified: (RPackage >> #toTagName:) protocol.

MCPackageLoader compile: 'tryToLoad: aDefinition
	(aDefinition isClassDefinition and: [
		aDefinition actualClass notNil ]) ifTrue: [ ^ false ].

	[aDefinition addMethodAdditionTo: methodAdditions] on: Error do: [errorDefinitions add: aDefinition].'
classified: (MCPackageLoader >> #tryToLoad:) protocol.

CompiledMethod compile: 'basicAsMCMethodDefinition
	^ MCMethodDefinition
		  className: self methodClass instanceSide name
		  classIsMeta: self isClassSide
		  selector: self selector
		  category: self protocol
		  timeStamp: '''' "self stamp"
		  source: self sourceCode'
classified: (CompiledMethod >> #basicAsMCMethodDefinition) protocol.

RBParser compile: 'parseKeywordMessageWith: node 
	| args isKeyword keywordsStartPositions selector selectorStream |
	args := OrderedCollection new: 3.
	keywordsStartPositions := OrderedCollection new: 3.
	selectorStream := WriteStream on: String new.
	isKeyword := false.
	[currentToken isKeyword] whileTrue: 
			[keywordsStartPositions add: currentToken start.
			selectorStream nextPutAll: currentToken value.
			self step.
			args add: self parseBinaryMessage.
			isKeyword := true].
	^isKeyword
		ifTrue: 
			[selector := self selectorNodeClass value: selectorStream contents.
			selector keywordPositions: keywordsStartPositions.
			self messageNodeClass 
				receiver: node
				selector: selector
				keywordsPositions: keywordsStartPositions
				arguments: args]
		ifFalse: [node]'
classified: (RBParser >> #parseKeywordMessageWith:) protocol.

ExternalData compile:
'readStringUTF8
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
classified: (ExternalData >> #readStringUTF8) protocol.

FFIUnixLibraryFinder compile:
'basePaths
	^ {
	(Smalltalk vm directory asFileReference parent / ''lib'') pathString.
	Smalltalk imageDirectory fullName.
	Smalltalk vm directory }'
classified: (FFIUnixLibraryFinder >> #basePaths) protocol.

LGitLibrary compile:
'macLibraryName

 	^ FFIMacLibraryFinder findAnyLibrary: #(''libgit2.dylib'' ''libgit2.1.0.1.dylib'' ''libgit2.1.0.0.dylib'' ''libgit2.0.25.1.dylib'')'
classified: (LGitLibrary >> #macLibraryName) protocol.

LGitLibrary compile:
'unix64LibraryName

	^ FFIUnix64LibraryFinder findAnyLibrary: #(
	    ''libgit2.so''
		"This name is wrong, but some versions of the VM has this library shipped with the bad name"
		''libgit2.1.0.0.so''
		''libgit2.so.1.0.0''
		''libgit2.so.1.0''
		''libgit2.so.1.1''
		''libgit2.so.0.25.1'')'
classified: (LGitLibrary >> #unix64LibraryName) protocol.

LGitLibrary compile:
'win32LibraryName

    ^ FFIWindowsLibraryFinder findAnyLibrary: #(''git2.dll'' ''libgit2-1-0-0.dll'' ''libgit2.dll'')'
classified: (LGitLibrary >> #win32LibraryName) protocol.

CairoLibrary compile:
'macLibraryName

 	^ FFIMacLibraryFinder findAnyLibrary: #(''libcairo.dylib'' ''libcairo.2.dylib'')'
classified: (CairoLibrary >> #macLibraryName) protocol.

CairoLibrary compile:
'win32LibraryName

 	^ FFIWindowsLibraryFinder findAnyLibrary: #(''cairo.dll'' ''libcairo-2.dll'')'
classified: (CairoLibrary >> #win32LibraryName) protocol.

ShMetaclassChanged compile:
'propagateToSubclasses: anotherBuilder
	anotherBuilder changes
		add: (self class new
				builder: anotherBuilder;
				yourself)'
classified: (ShMetaclassChanged >> #propagateToSubclasses:) protocol.

ShMetaclassChangeDetector compile:
'newChanges
	^ {ShMetaclassChanged new
			builder: builder;
			yourself.
		ShInstanceShapeChanged new
			builder: builder;
			yourself}'
classified: (ShMetaclassChangeDetector >> #newChanges) protocol.

RBCommentChange compile: 'primitiveExecute
	self changeClass classComment: comment stamp: self changeStamp.'
classified: (RBCommentChange >> #primitiveExecute) protocol.

WriteStream compile: 'pastEndPut: anObject
	"Grow the collection, then put <anObject> at the current write position."

	collection := collection grownBy: (collection size max: 20).
	writeLimit := collection size.
	collection at: (position := position + 1) put: anObject.
	^ anObject'
classified: (WriteStream >> #pastEndPut:) protocol.

ZnHTTPSTest compile: 'testGmailEncrypted
	| client |
	self ensureSocketStreamFactory.
	self isNativeSSLPluginPresent ifFalse: [ ^ self ].  
	
	(client := ZnClient new) 
		maxNumberOfRedirects: 10;
		get: ''https://www.gmail.com''.
	
	self assert: client isSuccess.
	self assert: (client contents includesSubstring: ''Google'').    
	self assert: (client contents includesSubstring: ''mail''). 
	client close'
classified: (ZnHTTPSTest >> #testGmailEncrypted) protocol.

ChunkWriteStream compile: 'afterNexPut 
	decoratedStream flush' classified: 'accessing'.
ChunkWriteStream compile: 'duplicateTerminatorMarkOn: aString 
	| string start bangIndex newStringStream |
	string := aString asString.
	newStringStream := WriteStream on: (string species new: string size * 2).
	start := 1.

	[ (bangIndex := string indexOf: self terminatorMark startingAt: start) = 0 ]
		whileFalse: [
			newStringStream
				next: bangIndex - start + 1
				putAll: string
				startingAt: start.

			newStringStream nextPut: self terminatorMark. 
			start := bangIndex + 1 ].

	newStringStream
		next: string size - start + 1
		putAll: string
		startingAt: start.

	newStringStream nextPut: self terminatorMark. 

	^ newStringStream contents' classified: 'accessing'.

SourceChunkWriteStream  compile: 'afterNexPut

	decoratedStream setToEnd' classified: 'accessing'.

ChunkWriteStream compile: 'nextPut: aString
	| string |
	string := self duplicateTerminatorMarkOn: aString asString.

	decoratedStream
		next: string size
		putAll: string
		startingAt: 1.

	self afterNexPut' classified: 'accessing'.
	
(SourceChunkWriteStream includesSelector: #nextPut:) ifTrue: [
	SourceChunkWriteStream removeSelector: #nextPut: ].

"Code to allow switch Iceberg repository implementations"
"----------------------"

Object subclass: #IceRepository
	instanceVariableNames: 'name workingCopy index commitsInPackageCache'
	classVariableNames: 'Registry RepositoryClass'
	package: 'Iceberg-Core'.

IceRepository class compile: 'repositoryClass: aClass
	RepositoryClass := aClass' classified: 'accessing'.

IceRepository class compile: 'repositoryClass
	^ RepositoryClass ifNil: [ RepositoryClass := IceLibgitRepository ]' classified: 'accessing'.

Iceberg class compile: 'bootstrapWithCommitId: commitId packageList: packageNameList

	| commit repository |
	repository := IceRepository repositoryClass new.
	commit := IceUnknownCommit new
		id: commitId;
		repository: repository;
 		yourself.
		
	repository name: ''iceberg''.
	repository workingCopy referenceCommit: commit.
	commit repository: repository.
	
	packageNameList do: [ :each |
		repository workingCopy basicAddPackage: (IcePackage
			named: each
			repository: repository) ].
	
	repository register' classified: 'accessing'.

IceRepositoryCreator compile: '
repository

	^ repository ifNil: [ repository := IceRepository repositoryClass new ]' classified: 'accessing'.

IceRepositoryCreator compile: 'createNewRepositoryNamed: aString
	| locationToUse  |
	self validate.
	locationToUse := self locationToUse.
	(locationToUse exists and: [ locationToUse hasChildren ])
		ifTrue: [ IceError
				signal: (''{1} is not empty.'' format: {locationToUse fullName}) ].
	locationToUse ensureCreateDirectory.
	self subdirectory
		ifNotEmpty: [ (locationToUse resolve: self subdirectory) ensureCreateDirectory ].
	repository := IceRepository repositoryClass new
		name: aString;
		location: locationToUse;
		initBare: isBareRepository;
		yourself.
	self addProjectFileTo: repository.
	self addPropertiesFileTo: repository.
	repository workingCopy refreshProject.
	^ repository' classified: 'actions'.

IcePharoPlugin class compile: 'addProjectNamed: aName commit: aCommitId baselines: aCollection
	| repository commit workingCopy |

	repository := IceRepository repositoryClass new
		name: aName;
		yourself.
	
	commit := IceUnknownCommit new 
		repository: repository;
		id: aCommitId;
	 	yourself.

	workingCopy := IceWorkingCopy basicNew
		repository: repository;
		initialize;
		referenceCommit: commit;
		project: ((IceBasicProject onRepository: repository)
			sourceDirectory: ''src'';
			yourself);
		yourself.

	repository workingCopy: workingCopy.
	
	(aCollection 
		flatCollect: [ :each | (Smalltalk globals at: each) allPackageNames copyWith: each ] 
		as: Set)
		do: [ :each | 
			repository workingCopy basicAddPackage: (IcePackage
				named: each
				repository: repository) ].

	repository register' classified: 'adding'.

IceMetacelloDuplicatedNotification compile: 'existingProjectRegistration
	| repository |
	repository := IceRepository repositoryClass new
		location: builder locationToUse.
	^ IceMetacelloProjectRegistration new
		version: repository head description;
		yourself' classified: 'accessing'.

"--------------------"

EpMonitor current enable.
Iceberg enableMetacelloIntegration: true.
