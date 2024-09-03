| externalAddressMethodsToPatch |

EpMonitor current disable.
Iceberg enableMetacelloIntegration: false.

NonInteractiveTranscript stdout
    nextPutAll: 'Patching Pharo Base image...';
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
		inClass: #ExternalAddress) asRefactoring execute.
		
	(RBAddPragmaTransformation
		pragma: '<gtPharoPatch: #Pharo>'
		inMethod: each
		inClass: #ExternalAddress) asRefactoring execute  ].

Object compile:
'setPinnedInMemory: aBoolean
 	"The VM''s garbage collector routinely moves objects as it reclaims and compacts
 	 memory. But it can also pin an object so that it will not be moved around in memory,
     while still being reclamable by the garbage collector. This can make
 	 it easier to pass objects out through the FFI. Objects are unpinnned when created.
 	 This primitive either pins or unpins an object, and answers if it was already pinned.

 	If there is not enough memory, I will try to find more memory and retry once."
 	<primitive: 184 error: ec>
	<gtPharoPatch: #Pharo>

 	ec = #''insufficient object memory''
 		ifFalse: [ ^ self primitiveFailed ].

 	Smalltalk garbageCollect < (self sizeInMemory * 2) ifTrue:
 		[Smalltalk growMemoryByAtLeast: self sizeInMemory * 2 ].

 	^ self retrySetPinnedInMemory: aBoolean'
classified: 'gt-pharo-patch'.

Object compile:
'asOrderedCollection
	<gtPharoPatch: #Pharo>

	^ OrderedCollection with: self'
classified: 'gt-pharo-patch'.

Metaclass compile:
'classVariableNamed: aString ifAbsent: absentBlock
	<gtPharoPatch: #Pharo>
	"Answer the Class Variable"

	^ self instanceSide
		ifNil: absentBlock
		ifNotNil: [ :class | class classVariableNamed: aString ifAbsent: absentBlock ]'
classified: 'gt-pharo-patch'.

Month class compile:
'indexOfMonth: aMonthName
	<gtPharoPatch: #Pharo>

	1 to: 12 do: [ :i |
		((MonthNames at: i) beginsWith: aMonthName caseSensitive: false) ifTrue: [^i] ].
 	self error: aMonthName , '' is not a recognized month name'''
classified: 'gt-pharo-patch'.

SystemVersion current major <= 11 ifTrue: [ 
RPackage compile: 'toTagName: aSymbol
	<gtPharoPatch: #Pharo>

	^ (aSymbol beginsWith: self name asString, ''-'')
		ifTrue: [ (aSymbol allButFirst: self name size + 1) asSymbol ]
		ifFalse: [ aSymbol ]'
classified: 'gt-pharo-patch'.

MCPackageLoader compile: 'tryToLoad: aDefinition
	<gtPharoPatch: #Pharo>

	(aDefinition isClassDefinition and: [
		aDefinition actualClass notNil ]) ifTrue: [ ^ false ].

	[aDefinition addMethodAdditionTo: methodAdditions] on: Error do: [errorDefinitions add: aDefinition].'
classified: 'gt-pharo-patch'.

CompiledMethod compile: 'basicAsMCMethodDefinition
	<gtPharoPatch: #Pharo>

	^ MCMethodDefinition
		  className: self methodClass instanceSide name
		  classIsMeta: self isClassSide
		  selector: self selector
		  category: self protocol
		  timeStamp: '''' "self stamp"
		  source: self sourceCode'
classified: 'gt-pharo-patch'.

MCMethodDefinition compile: 'addMethodAdditionTo: aCollection
	<gtPharoPatch: #Pharo>
	| methodAddition |
	methodAddition := MethodAddition new
		compile: source
		classified: category
		withStamp: timeStamp
		notifying: nil
		logSource: true
		inClass: self actualClass.
	aCollection add: methodAddition.'
classified: 'gt-pharo-patch'.

MethodAddition compile: 'compile
	<gtPharoPatch: #Pharo>
    "This method is the how compiling a method used to work.  All these steps were done at once.
     This method should not normally be used, because the whole point of MethodAddition is to let
	you first create a compiled method and then install the method later."
	self installMethod.
	self notifyObservers.
	^selector'
classified: 'gt-pharo-patch'.

MethodAddition compile: 'installMethod
	<gtPharoPatch: #Pharo>
	self createCompiledMethod.
	SystemAnnouncer uniqueInstance
		suspendAllWhile: [ myClass addSelector: selector withMethod: compiledMethod ]'
classified: 'gt-pharo-patch'.

 ].

SystemVersion current major >= 12 ifTrue: [ 

CompiledMethod compile: 'basicAsMCMethodDefinition
	<gtPharoPatch: #Pharo12>

	^ MCMethodDefinition
		  className: self methodClass instanceSide name
		  classIsMeta: self isClassSide
		  selector: self selector
		  category: self protocolName
		  timeStamp: '''' "self stamp"
		  source: self sourceCode'
classified: 'gt-pharo-patch' ].

RBParser compile: 'parseKeywordMessageWith: node 
	<gtPharoPatch: #Pharo>

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
classified: 'gt-pharo-patch'.

ExternalData compile:
'readStringUTF8
	<gtPharoPatch: #Pharo>

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
classified: 'gt-pharo-patch'.

FFIUnixLibraryFinder compile:
'basePaths
	<gtPharoPatch: #Pharo>

	^ {
	(Smalltalk vm directory asFileReference parent / ''lib'') pathString.
	Smalltalk imageDirectory fullName.
	Smalltalk vm directory }'
classified: 'gt-pharo-patch'.

LGitLibrary compile:
'macLibraryName
	<gtPharoPatch: #Pharo>

 	^ FFIMacLibraryFinder findAnyLibrary: #(''libgit2.dylib'' ''libgit2.1.0.1.dylib'' ''libgit2.1.0.0.dylib'' ''libgit2.0.25.1.dylib'')'
classified: 'gt-pharo-patch'.

LGitLibrary compile:
'unix64LibraryName
	<gtPharoPatch: #Pharo>

	^ FFIUnix64LibraryFinder findAnyLibrary: #(
	    ''libgit2.so''
		"This name is wrong, but some versions of the VM has this library shipped with the bad name"
		''libgit2.1.0.0.so''
		''libgit2.so.1.0.0''
		''libgit2.so.1.0''
		''libgit2.so.1.1''
		''libgit2.so.0.25.1'')'
classified: 'gt-pharo-patch'.

LGitLibrary compile:
'win32LibraryName
	<gtPharoPatch: #Pharo>

    ^ FFIWindowsLibraryFinder findAnyLibrary: #(''git2.dll'' ''libgit2-1-0-0.dll'' ''libgit2.dll'')'
classified: 'gt-pharo-patch'.

LGitExternalObject compile:
'withReturnHandlerDo: callBlock
	<gtPharoPatch: #Pharo>
	| retry |

	retry := 3.
	^ [ callBlock value handleLGitReturnCode ]
				on: Error
				do: [ :ex |
					(ex messageText indexOfSubCollection: ''Connection timed out'') = 0 ifTrue:
						[ ex pass ].
					retry := retry - 1.
					retry > 0 ifTrue:
						[ Stdio stdout 
							<< ''retry: ''; 
							<< retry asString; 
							<< '': '';
							<< ex printString;
							lf.
						ex retry ]
					ifFalse:
						[ ex pass ] ]'
classified: 'exit code handling'.

CairoLibrary compile:
'macLibraryName
	<gtPharoPatch: #Pharo>

 	^ FFIMacLibraryFinder findAnyLibrary: #(''libcairo.dylib'' ''libcairo.2.dylib'')'
classified: 'gt-pharo-patch'.

CairoLibrary compile:
'win32LibraryName
	<gtPharoPatch: #Pharo>

 	^ FFIWindowsLibraryFinder findAnyLibrary: #(''cairo.dll'' ''libcairo-2.dll'')'
classified: 'gt-pharo-patch'.

ShMetaclassChanged compile:
'propagateToSubclasses: anotherBuilder
	<gtPharoPatch: #Pharo>

	anotherBuilder changes
		add: (self class new
				builder: anotherBuilder;
				yourself)'
classified: 'gt-pharo-patch'.

ShMetaclassChangeDetector compile:
'newChanges
	<gtPharoPatch: #Pharo>

	^ {ShMetaclassChanged new
			builder: builder;
			yourself.
		ShInstanceShapeChanged new
			builder: builder;
			yourself}'
classified: 'gt-pharo-patch'.

SystemVersion current major <= 11 ifTrue: [ 
RBCommentChange compile: 'primitiveExecute
	<gtPharoPatch: #Pharo>

	self changeClass classComment: comment stamp: self changeStamp.'
classified: 'gt-pharo-patch' ].

WriteStream compile: 'pastEndPut: anObject
	<gtPharoPatch: #Pharo>

	"Grow the collection, then put <anObject> at the current write position."

	collection := collection grownBy: (collection size max: 20).
	writeLimit := collection size.
	collection at: (position := position + 1) put: anObject.
	^ anObject'
classified: 'gt-pharo-patch'.

ZnHTTPSTest compile: 'testGmailEncrypted
	<gtPharoPatch: #Pharo>

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
classified: 'gt-pharo-patch'.

ChunkWriteStream compile: 'afterNexPut 
	<gtPharoPatch: #Pharo>

	decoratedStream flush' classified: 'gt-pharo-patch'.
ChunkWriteStream compile: 'duplicateTerminatorMarkOn: aString 
	<gtPharoPatch: #Pharo>

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

	^ newStringStream contents' classified: 'gt-pharo-patch'.

SourceChunkWriteStream  compile: 'afterNexPut
	<gtPharoPatch: #Pharo>

	decoratedStream setToEnd' classified: 'gt-pharo-patch'.

ChunkWriteStream compile: 'nextPut: aString
	<gtPharoPatch: #Pharo>

	| string |
	string := self duplicateTerminatorMarkOn: aString asString.

	decoratedStream
		next: string size
		putAll: string
		startingAt: 1.

	self afterNexPut' classified: 'gt-pharo-patch'.
	
(SourceChunkWriteStream includesSelector: #nextPut:) ifTrue: [
	SourceChunkWriteStream removeSelector: #nextPut: ].

"Code to allow switch Iceberg repository implementations"
"----------------------"

Object subclass: #IceRepository
	instanceVariableNames: 'name workingCopy index commitsInPackageCache'
	classVariableNames: 'Registry RepositoryClass'
	package: 'Iceberg-Core'.

IceRepository class compile: 'repositoryClass: aClass
	<gtPharoPatch: #Pharo>

	RepositoryClass := aClass' classified: 'gt-pharo-patch'.

IceRepository class compile: 'repositoryClass
	<gtPharoPatch: #Pharo>

	^ RepositoryClass ifNil: [ RepositoryClass := IceLibgitRepository ]' classified: 'gt-pharo-patch'.

Iceberg class compile: 'bootstrapWithCommitId: commitId packageList: packageNameList
	<gtPharoPatch: #Pharo>

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
	
	repository register' classified: 'gt-pharo-patch'.

IcePharoPlugin class compile: 'addProjectNamed: aName commit: aCommitId baselines: aCollection
	<gtPharoPatch: #Pharo>

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

	repository register' classified: 'gt-pharo-patch'.

IceMetacelloDuplicatedNotification compile: 'existingProjectRegistration
	<gtPharoPatch: #Pharo>

	| repository |
	repository := IceRepository repositoryClass new
		location: builder locationToUse.
	^ IceMetacelloProjectRegistration new
		version: repository head description;
		yourself' classified: 'gt-pharo-patch'.

CompiledMethod compile: '= aCompiledMethod
	<gtPharoPatch: #Pharo>
	
	^ (super = aCompiledMethod) 
		ifTrue: [
			self selector = aCompiledMethod selector ]
		ifFalse: [ false ]' classified: 'gt-pharo-patch'.

ReflectiveMethod compile: 'installMethod: aMethod
	<gtPharoPatch: #Pharo>
	"add to method dictionary"

	| oldMethod |
	oldMethod := ast methodClass methodDict at: aMethod selector ifAbsent: [  ].
	ast methodClass methodDict at: aMethod selector put: aMethod.
	SystemAnnouncer uniqueInstance
		announce: (GtReflectiveMethodInstalled new
				oldMethod: oldMethod;
				newMethod: aMethod;
				yourself)'.

"--------------------"

EpMonitor current enable.
Iceberg enableMetacelloIntegration: true.
