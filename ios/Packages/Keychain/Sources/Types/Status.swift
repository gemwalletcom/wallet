// Copyright (c). Gem Wallet. All rights reserved.

import Foundation

enum Status: OSStatus, Error {
    case success = 0
    case unimplemented = -4
    case diskFull = -34
    case io = -36
    case opWr = -49
    case param = -50
    case wrPerm = -61
    case allocate = -108
    case userCanceled = -128
    case badReq = -909
    case internalComponent = -2070
    case notAvailable = -25291
    case readOnly = -25292
    case authFailed = -25293
    case noSuchKeychain = -25294
    case invalidKeychain = -25295
    case duplicateKeychain = -25296
    case duplicateCallback = -25297
    case invalidCallback = -25298
    case duplicateItem = -25299
    case itemNotFound = -25300
    case bufferTooSmall = -25301
    case dataTooLarge = -25302
    case noSuchAttr = -25303
    case invalidItemRef = -25304
    case invalidSearchRef = -25305
    case noSuchClass = -25306
    case noDefaultKeychain = -25307
    case interactionNotAllowed = -25308
    case readOnlyAttr = -25309
    case wrongSecVersion = -25310
    case keySizeNotAllowed = -25311
    case noStorageModule = -25312
    case noCertificateModule = -25313
    case noPolicyModule = -25314
    case interactionRequired = -25315
    case dataNotAvailable = -25316
    case dataNotModifiable = -25317
    case createChainFailed = -25318
    case invalidPrefsDomain = -25319
    case inDarkWake = -25320
    case aclNotSimple = -25240
    case policyNotFound = -25241
    case invalidTrustSetting = -25242
    case noAccessForItem = -25243
    case invalidOwnerEdit = -25244
    case trustNotAvailable = -25245
    case unsupportedFormat = -25256
    case unknownFormat = -25257
    case keyIsSensitive = -25258
    case multiplePrivKeys = -25259
    case passphraseRequired = -25260
    case invalidPasswordRef = -25261
    case invalidTrustSettings = -25262
    case noTrustSettings = -25263
    case pkcs12VerifyFailure = -25264
    case invalidCertificate = -26265
    case notSigner = -26267
    case policyDenied = -26270
    case invalidKey = -26274
    case decode = -26275
    case `internal` = -26276
    case unsupportedAlgorithm = -26268
    case unsupportedOperation = -26271
    case unsupportedPadding = -26273
    case itemInvalidKey = -34000
    case itemInvalidKeyType = -34001
    case itemInvalidValue = -34002
    case itemClassMissing = -34003
    case itemMatchUnsupported = -34004
    case useItemListUnsupported = -34005
    case useKeychainUnsupported = -34006
    case useKeychainListUnsupported = -34007
    case returnDataUnsupported = -34008
    case returnAttributesUnsupported = -34009
    case returnRefUnsupported = -34010
    case returnPersitentRefUnsupported = -34011
    case valueRefUnsupported = -34012
    case valuePersistentRefUnsupported = -34013
    case returnMissingPointer = -34014
    case matchLimitUnsupported = -34015
    case itemIllegalQuery = -34016
    case waitForCallback = -34017
    case missingEntitlement = -34018
    case upgradePending = -34019
    case mpSignatureInvalid = -25327
    case otrTooOld = -25328
    case otrIDTooNew = -25329
    case serviceNotAvailable = -67585
    case insufficientClientID = -67586
    case deviceReset = -67587
    case deviceFailed = -67588
    case appleAddAppACLSubject = -67589
    case applePublicKeyIncomplete = -67590
    case appleSignatureMismatch = -67591
    case appleInvalidKeyStartDate = -67592
    case appleInvalidKeyEndDate = -67593
    case conversionError = -67594
    case appleSSLv2Rollback = -67595
    case quotaExceeded = -67596
    case fileTooBig = -67597
    case invalidDatabaseBlob = -67598
    case invalidKeyBlob = -67599
    case incompatibleDatabaseBlob = -67600
    case incompatibleKeyBlob = -67601
    case hostNameMismatch = -67602
    case unknownCriticalExtensionFlag = -67603
    case noBasicConstraints = -67604
    case noBasicConstraintsCA = -67605
    case invalidAuthorityKeyID = -67606
    case invalidSubjectKeyID = -67607
    case invalidKeyUsageForPolicy = -67608
    case invalidExtendedKeyUsage = -67609
    case invalidIDLinkage = -67610
    case pathLengthConstraintExceeded = -67611
    case invalidRoot = -67612
    case crlExpired = -67613
    case crlNotValidYet = -67614
    case crlNotFound = -67615
    case crlServerDown = -67616
    case crlBadURI = -67617
    case unknownCertExtension = -67618
    case unknownCRLExtension = -67619
    case crlNotTrusted = -67620
    case crlPolicyFailed = -67621
    case idpFailure = -67622
    case smimeEmailAddressesNotFound = -67623
    case smimeBadExtendedKeyUsage = -67624
    case smimeBadKeyUsage = -67625
    case smimeKeyUsageNotCritical = -67626
    case smimeNoEmailAddress = -67627
    case smimeSubjAltNameNotCritical = -67628
    case sslBadExtendedKeyUsage = -67629
    case ocspBadResponse = -67630
    case ocspBadRequest = -67631
    case ocspUnavailable = -67632
    case ocspStatusUnrecognized = -67633
    case endOfData = -67634
    case incompleteCertRevocationCheck = -67635
    case networkFailure = -67636
    case ocspNotTrustedToAnchor = -67637
    case recordModified = -67638
    case ocspSignatureError = -67639
    case ocspNoSigner = -67640
    case ocspResponderMalformedReq = -67641
    case ocspResponderInternalError = -67642
    case ocspResponderTryLater = -67643
    case ocspResponderSignatureRequired = -67644
    case ocspResponderUnauthorized = -67645
    case ocspResponseNonceMismatch = -67646
    case codeSigningBadCertChainLength = -67647
    case codeSigningNoBasicConstraints = -67648
    case codeSigningBadPathLengthConstraint = -67649
    case codeSigningNoExtendedKeyUsage = -67650
    case codeSigningDevelopment = -67651
    case resourceSignBadCertChainLength = -67652
    case resourceSignBadExtKeyUsage = -67653
    case trustSettingDeny = -67654
    case invalidSubjectName = -67655
    case unknownQualifiedCertStatement = -67656
    case mobileMeRequestQueued = -67657
    case mobileMeRequestRedirected = -67658
    case mobileMeServerError = -67659
    case mobileMeServerNotAvailable = -67660
    case mobileMeServerAlreadyExists = -67661
    case mobileMeServerServiceErr = -67662
    case mobileMeRequestAlreadyPending = -67663
    case mobileMeNoRequestPending = -67664
    case mobileMeCSRVerifyFailure = -67665
    case mobileMeFailedConsistencyCheck = -67666
    case notInitialized = -67667
    case invalidHandleUsage = -67668
    case pvcReferentNotFound = -67669
    case functionIntegrityFail = -67670
    case internalError = -67671
    case memoryError = -67672
    case invalidData = -67673
    case mdsError = -67674
    case invalidPointer = -67675
    case selfCheckFailed = -67676
    case functionFailed = -67677
    case moduleManifestVerifyFailed = -67678
    case invalidGUID = -67679
    case invalidHandle = -67680
    case invalidDBList = -67681
    case invalidPassthroughID = -67682
    case invalidNetworkAddress = -67683
    case crlAlreadySigned = -67684
    case invalidNumberOfFields = -67685
    case verificationFailure = -67686
    case unknownTag = -67687
    case invalidSignature = -67688
    case invalidName = -67689
    case invalidCertificateRef = -67690
    case invalidCertificateGroup = -67691
    case tagNotFound = -67692
    case invalidQuery = -67693
    case invalidValue = -67694
    case callbackFailed = -67695
    case aclDeleteFailed = -67696
    case aclReplaceFailed = -67697
    case aclAddFailed = -67698
    case aclChangeFailed = -67699
    case invalidAccessCredentials = -67700
    case invalidRecord = -67701
    case invalidACL = -67702
    case invalidSampleValue = -67703
    case incompatibleVersion = -67704
    case privilegeNotGranted = -67705
    case invalidScope = -67706
    case pvcAlreadyConfigured = -67707
    case invalidPVC = -67708
    case emmLoadFailed = -67709
    case emmUnloadFailed = -67710
    case addinLoadFailed = -67711
    case invalidKeyRef = -67712
    case invalidKeyHierarchy = -67713
    case addinUnloadFailed = -67714
    case libraryReferenceNotFound = -67715
    case invalidAddinFunctionTable = -67716
    case invalidServiceMask = -67717
    case moduleNotLoaded = -67718
    case invalidSubServiceID = -67719
    case attributeNotInContext = -67720
    case moduleManagerInitializeFailed = -67721
    case moduleManagerNotFound = -67722
    case eventNotificationCallbackNotFound = -67723
    case inputLengthError = -67724
    case outputLengthError = -67725
    case privilegeNotSupported = -67726
    case deviceError = -67727
    case attachHandleBusy = -67728
    case notLoggedIn = -67729
    case algorithmMismatch = -67730
    case keyUsageIncorrect = -67731
    case keyBlobTypeIncorrect = -67732
    case keyHeaderInconsistent = -67733
    case unsupportedKeyFormat = -67734
    case unsupportedKeySize = -67735
    case invalidKeyUsageMask = -67736
    case unsupportedKeyUsageMask = -67737
    case invalidKeyAttributeMask = -67738
    case unsupportedKeyAttributeMask = -67739
    case invalidKeyLabel = -67740
    case unsupportedKeyLabel = -67741
    case invalidKeyFormat = -67742
    case unsupportedVectorOfBuffers = -67743
    case invalidInputVector = -67744
    case invalidOutputVector = -67745
    case invalidContext = -67746
    case invalidAlgorithm = -67747
    case invalidAttributeKey = -67748
    case missingAttributeKey = -67749
    case invalidAttributeInitVector = -67750
    case missingAttributeInitVector = -67751
    case invalidAttributeSalt = -67752
    case missingAttributeSalt = -67753
    case invalidAttributePadding = -67754
    case missingAttributePadding = -67755
    case invalidAttributeRandom = -67756
    case missingAttributeRandom = -67757
    case invalidAttributeSeed = -67758
    case missingAttributeSeed = -67759
    case invalidAttributePassphrase = -67760
    case missingAttributePassphrase = -67761
    case invalidAttributeKeyLength = -67762
    case missingAttributeKeyLength = -67763
    case invalidAttributeBlockSize = -67764
    case missingAttributeBlockSize = -67765
    case invalidAttributeOutputSize = -67766
    case missingAttributeOutputSize = -67767
    case invalidAttributeRounds = -67768
    case missingAttributeRounds = -67769
    case invalidAlgorithmParms = -67770
    case missingAlgorithmParms = -67771
    case invalidAttributeLabel = -67772
    case missingAttributeLabel = -67773
    case invalidAttributeKeyType = -67774
    case missingAttributeKeyType = -67775
    case invalidAttributeMode = -67776
    case missingAttributeMode = -67777
    case invalidAttributeEffectiveBits = -67778
    case missingAttributeEffectiveBits = -67779
    case invalidAttributeStartDate = -67780
    case missingAttributeStartDate = -67781
    case invalidAttributeEndDate = -67782
    case missingAttributeEndDate = -67783
    case invalidAttributeVersion = -67784
    case missingAttributeVersion = -67785
    case invalidAttributePrime = -67786
    case missingAttributePrime = -67787
    case invalidAttributeBase = -67788
    case missingAttributeBase = -67789
    case invalidAttributeSubprime = -67790
    case missingAttributeSubprime = -67791
    case invalidAttributeIterationCount = -67792
    case missingAttributeIterationCount = -67793
    case invalidAttributeDLDBHandle = -67794
    case missingAttributeDLDBHandle = -67795
    case invalidAttributeAccessCredentials = -67796
    case missingAttributeAccessCredentials = -67797
    case invalidAttributePublicKeyFormat = -67798
    case missingAttributePublicKeyFormat = -67799
    case invalidAttributePrivateKeyFormat = -67800
    case missingAttributePrivateKeyFormat = -67801
    case invalidAttributeSymmetricKeyFormat = -67802
    case missingAttributeSymmetricKeyFormat = -67803
    case invalidAttributeWrappedKeyFormat = -67804
    case missingAttributeWrappedKeyFormat = -67805
    case stagedOperationInProgress = -67806
    case stagedOperationNotStarted = -67807
    case verifyFailed = -67808
    case querySizeUnknown = -67809
    case blockSizeMismatch = -67810
    case publicKeyInconsistent = -67811
    case deviceVerifyFailed = -67812
    case invalidLoginName = -67813
    case alreadyLoggedIn = -67814
    case invalidDigestAlgorithm = -67815
    case invalidCRLGroup = -67816
    case certificateCannotOperate = -67817
    case certificateExpired = -67818
    case certificateNotValidYet = -67819
    case certificateRevoked = -67820
    case certificateSuspended = -67821
    case insufficientCredentials = -67822
    case invalidAction = -67823
    case invalidAuthority = -67824
    case verifyActionFailed = -67825
    case invalidCertAuthority = -67826
    case invaldCRLAuthority = -67827
    case invalidCRLEncoding = -67828
    case invalidCRLType = -67829
    case invalidCRL = -67830
    case invalidFormType = -67831
    case invalidID = -67832
    case invalidIdentifier = -67833
    case invalidIndex = -67834
    case invalidPolicyIdentifiers = -67835
    case invalidTimeString = -67836
    case invalidReason = -67837
    case invalidRequestInputs = -67838
    case invalidResponseVector = -67839
    case invalidStopOnPolicy = -67840
    case invalidTuple = -67841
    case multipleValuesUnsupported = -67842
    case notTrusted = -67843
    case noDefaultAuthority = -67844
    case rejectedForm = -67845
    case requestLost = -67846
    case requestRejected = -67847
    case unsupportedAddressType = -67848
    case unsupportedService = -67849
    case invalidTupleGroup = -67850
    case invalidBaseACLs = -67851
    case invalidTupleCredendtials = -67852
    case invalidEncoding = -67853
    case invalidValidityPeriod = -67854
    case invalidRequestor = -67855
    case requestDescriptor = -67856
    case invalidBundleInfo = -67857
    case invalidCRLIndex = -67858
    case noFieldValues = -67859
    case unsupportedFieldFormat = -67860
    case unsupportedIndexInfo = -67861
    case unsupportedLocality = -67862
    case unsupportedNumAttributes = -67863
    case unsupportedNumIndexes = -67864
    case unsupportedNumRecordTypes = -67865
    case fieldSpecifiedMultiple = -67866
    case incompatibleFieldFormat = -67867
    case invalidParsingModule = -67868
    case databaseLocked = -67869
    case datastoreIsOpen = -67870
    case missingValue = -67871
    case unsupportedQueryLimits = -67872
    case unsupportedNumSelectionPreds = -67873
    case unsupportedOperator = -67874
    case invalidDBLocation = -67875
    case invalidAccessRequest = -67876
    case invalidIndexInfo = -67877
    case invalidNewOwner = -67878
    case invalidModifyMode = -67879
    case missingRequiredExtension = -67880
    case extendedKeyUsageNotCritical = -67881
    case timestampMissing = -67882
    case timestampInvalid = -67883
    case timestampNotTrusted = -67884
    case timestampServiceNotAvailable = -67885
    case timestampBadAlg = -67886
    case timestampBadRequest = -67887
    case timestampBadDataFormat = -67888
    case timestampTimeNotAvailable = -67889
    case timestampUnacceptedPolicy = -67890
    case timestampUnacceptedExtension = -67891
    case timestampAddInfoNotAvailable = -67892
    case timestampSystemFailure = -67893
    case signingTimeMissing = -67894
    case timestampRejection = -67895
    case timestampWaiting = -67896
    case timestampRevocationWarning = -67897
    case timestampRevocationNotification = -67898
    case unexpectedError = -99999
}

extension Status: RawRepresentable, CustomStringConvertible {
    init(status: OSStatus) {
        if let mappedStatus = Status(rawValue: status) {
            self = mappedStatus
        } else {
            self = .unexpectedError
        }
    }

    var description: String {
        switch self {
        case .success:
            "No error."
        case .unimplemented:
            "Function or operation not implemented."
        case .diskFull:
            "The disk is full."
        case .io:
            "I/O error (bummers)"
        case .opWr:
            "file already open with with write permission"
        case .param:
            "One or more parameters passed to a function were not valid."
        case .wrPerm:
            "write permissions error"
        case .allocate:
            "Failed to allocate memory."
        case .userCanceled:
            "User canceled the operation."
        case .badReq:
            "Bad parameter or invalid state for operation."
        case .internalComponent:
            ""
        case .notAvailable:
            "No keychain is available. You may need to restart your computer."
        case .readOnly:
            "This keychain cannot be modified."
        case .authFailed:
            "The user name or passphrase you entered is not correct."
        case .noSuchKeychain:
            "The specified keychain could not be found."
        case .invalidKeychain:
            "The specified keychain is not a valid keychain file."
        case .duplicateKeychain:
            "A keychain with the same name already exists."
        case .duplicateCallback:
            "The specified callback function is already installed."
        case .invalidCallback:
            "The specified callback function is not valid."
        case .duplicateItem:
            "The specified item already exists in the keychain."
        case .itemNotFound:
            "The specified item could not be found in the keychain."
        case .bufferTooSmall:
            "There is not enough memory available to use the specified item."
        case .dataTooLarge:
            "This item contains information which is too large or in a format that cannot be displayed."
        case .noSuchAttr:
            "The specified attribute does not exist."
        case .invalidItemRef:
            "The specified item is no longer valid. It may have been deleted from the keychain."
        case .invalidSearchRef:
            "Unable to search the current keychain."
        case .noSuchClass:
            "The specified item does not appear to be a valid keychain item."
        case .noDefaultKeychain:
            "A default keychain could not be found."
        case .interactionNotAllowed:
            "User interaction is not allowed."
        case .readOnlyAttr:
            "The specified attribute could not be modified."
        case .wrongSecVersion:
            "This keychain was created by a different version of the system software and cannot be opened."
        case .keySizeNotAllowed:
            "This item specifies a key size which is too large."
        case .noStorageModule:
            "A required component (data storage module) could not be loaded. You may need to restart your computer."
        case .noCertificateModule:
            "A required component (certificate module) could not be loaded. You may need to restart your computer."
        case .noPolicyModule:
            "A required component (policy module) could not be loaded. You may need to restart your computer."
        case .interactionRequired:
            "User interaction is required, but is currently not allowed."
        case .dataNotAvailable:
            "The contents of this item cannot be retrieved."
        case .dataNotModifiable:
            "The contents of this item cannot be modified."
        case .createChainFailed:
            "One or more certificates required to validate this certificate cannot be found."
        case .invalidPrefsDomain:
            "The specified preferences domain is not valid."
        case .inDarkWake:
            "In dark wake, no UI possible"
        case .aclNotSimple:
            "The specified access control list is not in standard (simple) form."
        case .policyNotFound:
            "The specified policy cannot be found."
        case .invalidTrustSetting:
            "The specified trust setting is invalid."
        case .noAccessForItem:
            "The specified item has no access control."
        case .invalidOwnerEdit:
            "Invalid attempt to change the owner of this item."
        case .trustNotAvailable:
            "No trust results are available."
        case .unsupportedFormat:
            "Import/Export format unsupported."
        case .unknownFormat:
            "Unknown format in import."
        case .keyIsSensitive:
            "Key material must be wrapped for export."
        case .multiplePrivKeys:
            "An attempt was made to import multiple private keys."
        case .passphraseRequired:
            "Passphrase is required for import/export."
        case .invalidPasswordRef:
            "The password reference was invalid."
        case .invalidTrustSettings:
            "The Trust Settings Record was corrupted."
        case .noTrustSettings:
            "No Trust Settings were found."
        case .pkcs12VerifyFailure:
            "MAC verification failed during PKCS12 import (wrong password?)"
        case .invalidCertificate:
            "This certificate could not be decoded."
        case .notSigner:
            "A certificate was not signed by its proposed parent."
        case .policyDenied:
            "The certificate chain was not trusted due to a policy not accepting it."
        case .invalidKey:
            "The provided key material was not valid."
        case .decode:
            "Unable to decode the provided data."
        case .internal:
            "An internal error occurred in the Security framework."
        case .unsupportedAlgorithm:
            "An unsupported algorithm was encountered."
        case .unsupportedOperation:
            "The operation you requested is not supported by this key."
        case .unsupportedPadding:
            "The padding you requested is not supported."
        case .itemInvalidKey:
            "A string key in dictionary is not one of the supported keys."
        case .itemInvalidKeyType:
            "A key in a dictionary is neither a CFStringRef nor a CFNumberRef."
        case .itemInvalidValue:
            "A value in a dictionary is an invalid (or unsupported) CF type."
        case .itemClassMissing:
            "No kSecItemClass key was specified in a dictionary."
        case .itemMatchUnsupported:
            "The caller passed one or more kSecMatch keys to a function which does not support matches."
        case .useItemListUnsupported:
            "The caller passed in a kSecUseItemList key to a function which does not support it."
        case .useKeychainUnsupported:
            "The caller passed in a kSecUseKeychain key to a function which does not support it."
        case .useKeychainListUnsupported:
            "The caller passed in a kSecUseKeychainList key to a function which does not support it."
        case .returnDataUnsupported:
            "The caller passed in a kSecReturnData key to a function which does not support it."
        case .returnAttributesUnsupported:
            "The caller passed in a kSecReturnAttributes key to a function which does not support it."
        case .returnRefUnsupported:
            "The caller passed in a kSecReturnRef key to a function which does not support it."
        case .returnPersitentRefUnsupported:
            "The caller passed in a kSecReturnPersistentRef key to a function which does not support it."
        case .valueRefUnsupported:
            "The caller passed in a kSecValueRef key to a function which does not support it."
        case .valuePersistentRefUnsupported:
            "The caller passed in a kSecValuePersistentRef key to a function which does not support it."
        case .returnMissingPointer:
            "The caller passed asked for something to be returned but did not pass in a result pointer."
        case .matchLimitUnsupported:
            "The caller passed in a kSecMatchLimit key to a call which does not support limits."
        case .itemIllegalQuery:
            "The caller passed in a query which contained too many keys."
        case .waitForCallback:
            "This operation is incomplete, until the callback is invoked (not an error)."
        case .missingEntitlement:
            "Internal error when a required entitlement isn't present, client has neither application-identifier nor keychain-access-groups entitlements."
        case .upgradePending:
            "Error returned if keychain database needs a schema migration but the device is locked, clients should wait for a device unlock notification and retry the command."
        case .mpSignatureInvalid:
            "Signature invalid on MP message"
        case .otrTooOld:
            "Message is too old to use"
        case .otrIDTooNew:
            "Key ID is too new to use! Message from the future?"
        case .serviceNotAvailable:
            "The required service is not available."
        case .insufficientClientID:
            "The client ID is not correct."
        case .deviceReset:
            "A device reset has occurred."
        case .deviceFailed:
            "A device failure has occurred."
        case .appleAddAppACLSubject:
            "Adding an application ACL subject failed."
        case .applePublicKeyIncomplete:
            "The public key is incomplete."
        case .appleSignatureMismatch:
            "A signature mismatch has occurred."
        case .appleInvalidKeyStartDate:
            "The specified key has an invalid start date."
        case .appleInvalidKeyEndDate:
            "The specified key has an invalid end date."
        case .conversionError:
            "A conversion error has occurred."
        case .appleSSLv2Rollback:
            "A SSLv2 rollback error has occurred."
        case .quotaExceeded:
            "The quota was exceeded."
        case .fileTooBig:
            "The file is too big."
        case .invalidDatabaseBlob:
            "The specified database has an invalid blob."
        case .invalidKeyBlob:
            "The specified database has an invalid key blob."
        case .incompatibleDatabaseBlob:
            "The specified database has an incompatible blob."
        case .incompatibleKeyBlob:
            "The specified database has an incompatible key blob."
        case .hostNameMismatch:
            "A host name mismatch has occurred."
        case .unknownCriticalExtensionFlag:
            "There is an unknown critical extension flag."
        case .noBasicConstraints:
            "No basic constraints were found."
        case .noBasicConstraintsCA:
            "No basic CA constraints were found."
        case .invalidAuthorityKeyID:
            "The authority key ID is not valid."
        case .invalidSubjectKeyID:
            "The subject key ID is not valid."
        case .invalidKeyUsageForPolicy:
            "The key usage is not valid for the specified policy."
        case .invalidExtendedKeyUsage:
            "The extended key usage is not valid."
        case .invalidIDLinkage:
            "The ID linkage is not valid."
        case .pathLengthConstraintExceeded:
            "The path length constraint was exceeded."
        case .invalidRoot:
            "The root or anchor certificate is not valid."
        case .crlExpired:
            "The CRL has expired."
        case .crlNotValidYet:
            "The CRL is not yet valid."
        case .crlNotFound:
            "The CRL was not found."
        case .crlServerDown:
            "The CRL server is down."
        case .crlBadURI:
            "The CRL has a bad Uniform Resource Identifier."
        case .unknownCertExtension:
            "An unknown certificate extension was encountered."
        case .unknownCRLExtension:
            "An unknown CRL extension was encountered."
        case .crlNotTrusted:
            "The CRL is not trusted."
        case .crlPolicyFailed:
            "The CRL policy failed."
        case .idpFailure:
            "The issuing distribution point was not valid."
        case .smimeEmailAddressesNotFound:
            "An email address mismatch was encountered."
        case .smimeBadExtendedKeyUsage:
            "The appropriate extended key usage for SMIME was not found."
        case .smimeBadKeyUsage:
            "The key usage is not compatible with SMIME."
        case .smimeKeyUsageNotCritical:
            "The key usage extension is not marked as critical."
        case .smimeNoEmailAddress:
            "No email address was found in the certificate."
        case .smimeSubjAltNameNotCritical:
            "The subject alternative name extension is not marked as critical."
        case .sslBadExtendedKeyUsage:
            "The appropriate extended key usage for SSL was not found."
        case .ocspBadResponse:
            "The OCSP response was incorrect or could not be parsed."
        case .ocspBadRequest:
            "The OCSP request was incorrect or could not be parsed."
        case .ocspUnavailable:
            "OCSP service is unavailable."
        case .ocspStatusUnrecognized:
            "The OCSP server did not recognize this certificate."
        case .endOfData:
            "An end-of-data was detected."
        case .incompleteCertRevocationCheck:
            "An incomplete certificate revocation check occurred."
        case .networkFailure:
            "A network failure occurred."
        case .ocspNotTrustedToAnchor:
            "The OCSP response was not trusted to a root or anchor certificate."
        case .recordModified:
            "The record was modified."
        case .ocspSignatureError:
            "The OCSP response had an invalid signature."
        case .ocspNoSigner:
            "The OCSP response had no signer."
        case .ocspResponderMalformedReq:
            "The OCSP responder was given a malformed request."
        case .ocspResponderInternalError:
            "The OCSP responder encountered an internal error."
        case .ocspResponderTryLater:
            "The OCSP responder is busy, try again later."
        case .ocspResponderSignatureRequired:
            "The OCSP responder requires a signature."
        case .ocspResponderUnauthorized:
            "The OCSP responder rejected this request as unauthorized."
        case .ocspResponseNonceMismatch:
            "The OCSP response nonce did not match the request."
        case .codeSigningBadCertChainLength:
            "Code signing encountered an incorrect certificate chain length."
        case .codeSigningNoBasicConstraints:
            "Code signing found no basic constraints."
        case .codeSigningBadPathLengthConstraint:
            "Code signing encountered an incorrect path length constraint."
        case .codeSigningNoExtendedKeyUsage:
            "Code signing found no extended key usage."
        case .codeSigningDevelopment:
            "Code signing indicated use of a development-only certificate."
        case .resourceSignBadCertChainLength:
            "Resource signing has encountered an incorrect certificate chain length."
        case .resourceSignBadExtKeyUsage:
            "Resource signing has encountered an error in the extended key usage."
        case .trustSettingDeny:
            "The trust setting for this policy was set to Deny."
        case .invalidSubjectName:
            "An invalid certificate subject name was encountered."
        case .unknownQualifiedCertStatement:
            "An unknown qualified certificate statement was encountered."
        case .mobileMeRequestQueued:
            "The MobileMe request will be sent during the next connection."
        case .mobileMeRequestRedirected:
            "The MobileMe request was redirected."
        case .mobileMeServerError:
            "A MobileMe server error occurred."
        case .mobileMeServerNotAvailable:
            "The MobileMe server is not available."
        case .mobileMeServerAlreadyExists:
            "The MobileMe server reported that the item already exists."
        case .mobileMeServerServiceErr:
            "A MobileMe service error has occurred."
        case .mobileMeRequestAlreadyPending:
            "A MobileMe request is already pending."
        case .mobileMeNoRequestPending:
            "MobileMe has no request pending."
        case .mobileMeCSRVerifyFailure:
            "A MobileMe CSR verification failure has occurred."
        case .mobileMeFailedConsistencyCheck:
            "MobileMe has found a failed consistency check."
        case .notInitialized:
            "A function was called without initializing CSSM."
        case .invalidHandleUsage:
            "The CSSM handle does not match with the service type."
        case .pvcReferentNotFound:
            "A reference to the calling module was not found in the list of authorized callers."
        case .functionIntegrityFail:
            "A function address was not within the verified module."
        case .internalError:
            "An internal error has occurred."
        case .memoryError:
            "A memory error has occurred."
        case .invalidData:
            "Invalid data was encountered."
        case .mdsError:
            "A Module Directory Service error has occurred."
        case .invalidPointer:
            "An invalid pointer was encountered."
        case .selfCheckFailed:
            "Self-check has failed."
        case .functionFailed:
            "A function has failed."
        case .moduleManifestVerifyFailed:
            "A module manifest verification failure has occurred."
        case .invalidGUID:
            "An invalid GUID was encountered."
        case .invalidHandle:
            "An invalid handle was encountered."
        case .invalidDBList:
            "An invalid DB list was encountered."
        case .invalidPassthroughID:
            "An invalid passthrough ID was encountered."
        case .invalidNetworkAddress:
            "An invalid network address was encountered."
        case .crlAlreadySigned:
            "The certificate revocation list is already signed."
        case .invalidNumberOfFields:
            "An invalid number of fields were encountered."
        case .verificationFailure:
            "A verification failure occurred."
        case .unknownTag:
            "An unknown tag was encountered."
        case .invalidSignature:
            "An invalid signature was encountered."
        case .invalidName:
            "An invalid name was encountered."
        case .invalidCertificateRef:
            "An invalid certificate reference was encountered."
        case .invalidCertificateGroup:
            "An invalid certificate group was encountered."
        case .tagNotFound:
            "The specified tag was not found."
        case .invalidQuery:
            "The specified query was not valid."
        case .invalidValue:
            "An invalid value was detected."
        case .callbackFailed:
            "A callback has failed."
        case .aclDeleteFailed:
            "An ACL delete operation has failed."
        case .aclReplaceFailed:
            "An ACL replace operation has failed."
        case .aclAddFailed:
            "An ACL add operation has failed."
        case .aclChangeFailed:
            "An ACL change operation has failed."
        case .invalidAccessCredentials:
            "Invalid access credentials were encountered."
        case .invalidRecord:
            "An invalid record was encountered."
        case .invalidACL:
            "An invalid ACL was encountered."
        case .invalidSampleValue:
            "An invalid sample value was encountered."
        case .incompatibleVersion:
            "An incompatible version was encountered."
        case .privilegeNotGranted:
            "The privilege was not granted."
        case .invalidScope:
            "An invalid scope was encountered."
        case .pvcAlreadyConfigured:
            "The PVC is already configured."
        case .invalidPVC:
            "An invalid PVC was encountered."
        case .emmLoadFailed:
            "The EMM load has failed."
        case .emmUnloadFailed:
            "The EMM unload has failed."
        case .addinLoadFailed:
            "The add-in load operation has failed."
        case .invalidKeyRef:
            "An invalid key was encountered."
        case .invalidKeyHierarchy:
            "An invalid key hierarchy was encountered."
        case .addinUnloadFailed:
            "The add-in unload operation has failed."
        case .libraryReferenceNotFound:
            "A library reference was not found."
        case .invalidAddinFunctionTable:
            "An invalid add-in function table was encountered."
        case .invalidServiceMask:
            "An invalid service mask was encountered."
        case .moduleNotLoaded:
            "A module was not loaded."
        case .invalidSubServiceID:
            "An invalid subservice ID was encountered."
        case .attributeNotInContext:
            "An attribute was not in the context."
        case .moduleManagerInitializeFailed:
            "A module failed to initialize."
        case .moduleManagerNotFound:
            "A module was not found."
        case .eventNotificationCallbackNotFound:
            "An event notification callback was not found."
        case .inputLengthError:
            "An input length error was encountered."
        case .outputLengthError:
            "An output length error was encountered."
        case .privilegeNotSupported:
            "The privilege is not supported."
        case .deviceError:
            "A device error was encountered."
        case .attachHandleBusy:
            "The CSP handle was busy."
        case .notLoggedIn:
            "You are not logged in."
        case .algorithmMismatch:
            "An algorithm mismatch was encountered."
        case .keyUsageIncorrect:
            "The key usage is incorrect."
        case .keyBlobTypeIncorrect:
            "The key blob type is incorrect."
        case .keyHeaderInconsistent:
            "The key header is inconsistent."
        case .unsupportedKeyFormat:
            "The key header format is not supported."
        case .unsupportedKeySize:
            "The key size is not supported."
        case .invalidKeyUsageMask:
            "The key usage mask is not valid."
        case .unsupportedKeyUsageMask:
            "The key usage mask is not supported."
        case .invalidKeyAttributeMask:
            "The key attribute mask is not valid."
        case .unsupportedKeyAttributeMask:
            "The key attribute mask is not supported."
        case .invalidKeyLabel:
            "The key label is not valid."
        case .unsupportedKeyLabel:
            "The key label is not supported."
        case .invalidKeyFormat:
            "The key format is not valid."
        case .unsupportedVectorOfBuffers:
            "The vector of buffers is not supported."
        case .invalidInputVector:
            "The input vector is not valid."
        case .invalidOutputVector:
            "The output vector is not valid."
        case .invalidContext:
            "An invalid context was encountered."
        case .invalidAlgorithm:
            "An invalid algorithm was encountered."
        case .invalidAttributeKey:
            "A key attribute was not valid."
        case .missingAttributeKey:
            "A key attribute was missing."
        case .invalidAttributeInitVector:
            "An init vector attribute was not valid."
        case .missingAttributeInitVector:
            "An init vector attribute was missing."
        case .invalidAttributeSalt:
            "A salt attribute was not valid."
        case .missingAttributeSalt:
            "A salt attribute was missing."
        case .invalidAttributePadding:
            "A padding attribute was not valid."
        case .missingAttributePadding:
            "A padding attribute was missing."
        case .invalidAttributeRandom:
            "A random number attribute was not valid."
        case .missingAttributeRandom:
            "A random number attribute was missing."
        case .invalidAttributeSeed:
            "A seed attribute was not valid."
        case .missingAttributeSeed:
            "A seed attribute was missing."
        case .invalidAttributePassphrase:
            "A passphrase attribute was not valid."
        case .missingAttributePassphrase:
            "A passphrase attribute was missing."
        case .invalidAttributeKeyLength:
            "A key length attribute was not valid."
        case .missingAttributeKeyLength:
            "A key length attribute was missing."
        case .invalidAttributeBlockSize:
            "A block size attribute was not valid."
        case .missingAttributeBlockSize:
            "A block size attribute was missing."
        case .invalidAttributeOutputSize:
            "An output size attribute was not valid."
        case .missingAttributeOutputSize:
            "An output size attribute was missing."
        case .invalidAttributeRounds:
            "The number of rounds attribute was not valid."
        case .missingAttributeRounds:
            "The number of rounds attribute was missing."
        case .invalidAlgorithmParms:
            "An algorithm parameters attribute was not valid."
        case .missingAlgorithmParms:
            "An algorithm parameters attribute was missing."
        case .invalidAttributeLabel:
            "A label attribute was not valid."
        case .missingAttributeLabel:
            "A label attribute was missing."
        case .invalidAttributeKeyType:
            "A key type attribute was not valid."
        case .missingAttributeKeyType:
            "A key type attribute was missing."
        case .invalidAttributeMode:
            "A mode attribute was not valid."
        case .missingAttributeMode:
            "A mode attribute was missing."
        case .invalidAttributeEffectiveBits:
            "An effective bits attribute was not valid."
        case .missingAttributeEffectiveBits:
            "An effective bits attribute was missing."
        case .invalidAttributeStartDate:
            "A start date attribute was not valid."
        case .missingAttributeStartDate:
            "A start date attribute was missing."
        case .invalidAttributeEndDate:
            "An end date attribute was not valid."
        case .missingAttributeEndDate:
            "An end date attribute was missing."
        case .invalidAttributeVersion:
            "A version attribute was not valid."
        case .missingAttributeVersion:
            "A version attribute was missing."
        case .invalidAttributePrime:
            "A prime attribute was not valid."
        case .missingAttributePrime:
            "A prime attribute was missing."
        case .invalidAttributeBase:
            "A base attribute was not valid."
        case .missingAttributeBase:
            "A base attribute was missing."
        case .invalidAttributeSubprime:
            "A subprime attribute was not valid."
        case .missingAttributeSubprime:
            "A subprime attribute was missing."
        case .invalidAttributeIterationCount:
            "An iteration count attribute was not valid."
        case .missingAttributeIterationCount:
            "An iteration count attribute was missing."
        case .invalidAttributeDLDBHandle:
            "A database handle attribute was not valid."
        case .missingAttributeDLDBHandle:
            "A database handle attribute was missing."
        case .invalidAttributeAccessCredentials:
            "An access credentials attribute was not valid."
        case .missingAttributeAccessCredentials:
            "An access credentials attribute was missing."
        case .invalidAttributePublicKeyFormat:
            "A public key format attribute was not valid."
        case .missingAttributePublicKeyFormat:
            "A public key format attribute was missing."
        case .invalidAttributePrivateKeyFormat:
            "A private key format attribute was not valid."
        case .missingAttributePrivateKeyFormat:
            "A private key format attribute was missing."
        case .invalidAttributeSymmetricKeyFormat:
            "A symmetric key format attribute was not valid."
        case .missingAttributeSymmetricKeyFormat:
            "A symmetric key format attribute was missing."
        case .invalidAttributeWrappedKeyFormat:
            "A wrapped key format attribute was not valid."
        case .missingAttributeWrappedKeyFormat:
            "A wrapped key format attribute was missing."
        case .stagedOperationInProgress:
            "A staged operation is in progress."
        case .stagedOperationNotStarted:
            "A staged operation was not started."
        case .verifyFailed:
            "A cryptographic verification failure has occurred."
        case .querySizeUnknown:
            "The query size is unknown."
        case .blockSizeMismatch:
            "A block size mismatch occurred."
        case .publicKeyInconsistent:
            "The public key was inconsistent."
        case .deviceVerifyFailed:
            "A device verification failure has occurred."
        case .invalidLoginName:
            "An invalid login name was detected."
        case .alreadyLoggedIn:
            "The user is already logged in."
        case .invalidDigestAlgorithm:
            "An invalid digest algorithm was detected."
        case .invalidCRLGroup:
            "An invalid CRL group was detected."
        case .certificateCannotOperate:
            "The certificate cannot operate."
        case .certificateExpired:
            "An expired certificate was detected."
        case .certificateNotValidYet:
            "The certificate is not yet valid."
        case .certificateRevoked:
            "The certificate was revoked."
        case .certificateSuspended:
            "The certificate was suspended."
        case .insufficientCredentials:
            "Insufficient credentials were detected."
        case .invalidAction:
            "The action was not valid."
        case .invalidAuthority:
            "The authority was not valid."
        case .verifyActionFailed:
            "A verify action has failed."
        case .invalidCertAuthority:
            "The certificate authority was not valid."
        case .invaldCRLAuthority:
            "The CRL authority was not valid."
        case .invalidCRLEncoding:
            "The CRL encoding was not valid."
        case .invalidCRLType:
            "The CRL type was not valid."
        case .invalidCRL:
            "The CRL was not valid."
        case .invalidFormType:
            "The form type was not valid."
        case .invalidID:
            "The ID was not valid."
        case .invalidIdentifier:
            "The identifier was not valid."
        case .invalidIndex:
            "The index was not valid."
        case .invalidPolicyIdentifiers:
            "The policy identifiers are not valid."
        case .invalidTimeString:
            "The time specified was not valid."
        case .invalidReason:
            "The trust policy reason was not valid."
        case .invalidRequestInputs:
            "The request inputs are not valid."
        case .invalidResponseVector:
            "The response vector was not valid."
        case .invalidStopOnPolicy:
            "The stop-on policy was not valid."
        case .invalidTuple:
            "The tuple was not valid."
        case .multipleValuesUnsupported:
            "Multiple values are not supported."
        case .notTrusted:
            "The trust policy was not trusted."
        case .noDefaultAuthority:
            "No default authority was detected."
        case .rejectedForm:
            "The trust policy had a rejected form."
        case .requestLost:
            "The request was lost."
        case .requestRejected:
            "The request was rejected."
        case .unsupportedAddressType:
            "The address type is not supported."
        case .unsupportedService:
            "The service is not supported."
        case .invalidTupleGroup:
            "The tuple group was not valid."
        case .invalidBaseACLs:
            "The base ACLs are not valid."
        case .invalidTupleCredendtials:
            "The tuple credentials are not valid."
        case .invalidEncoding:
            "The encoding was not valid."
        case .invalidValidityPeriod:
            "The validity period was not valid."
        case .invalidRequestor:
            "The requestor was not valid."
        case .requestDescriptor:
            "The request descriptor was not valid."
        case .invalidBundleInfo:
            "The bundle information was not valid."
        case .invalidCRLIndex:
            "The CRL index was not valid."
        case .noFieldValues:
            "No field values were detected."
        case .unsupportedFieldFormat:
            "The field format is not supported."
        case .unsupportedIndexInfo:
            "The index information is not supported."
        case .unsupportedLocality:
            "The locality is not supported."
        case .unsupportedNumAttributes:
            "The number of attributes is not supported."
        case .unsupportedNumIndexes:
            "The number of indexes is not supported."
        case .unsupportedNumRecordTypes:
            "The number of record types is not supported."
        case .fieldSpecifiedMultiple:
            "Too many fields were specified."
        case .incompatibleFieldFormat:
            "The field format was incompatible."
        case .invalidParsingModule:
            "The parsing module was not valid."
        case .databaseLocked:
            "The database is locked."
        case .datastoreIsOpen:
            "The data store is open."
        case .missingValue:
            "A missing value was detected."
        case .unsupportedQueryLimits:
            "The query limits are not supported."
        case .unsupportedNumSelectionPreds:
            "The number of selection predicates is not supported."
        case .unsupportedOperator:
            "The operator is not supported."
        case .invalidDBLocation:
            "The database location is not valid."
        case .invalidAccessRequest:
            "The access request is not valid."
        case .invalidIndexInfo:
            "The index information is not valid."
        case .invalidNewOwner:
            "The new owner is not valid."
        case .invalidModifyMode:
            "The modify mode is not valid."
        case .missingRequiredExtension:
            "A required certificate extension is missing."
        case .extendedKeyUsageNotCritical:
            "The extended key usage extension was not marked critical."
        case .timestampMissing:
            "A timestamp was expected but was not found."
        case .timestampInvalid:
            "The timestamp was not valid."
        case .timestampNotTrusted:
            "The timestamp was not trusted."
        case .timestampServiceNotAvailable:
            "The timestamp service is not available."
        case .timestampBadAlg:
            "An unrecognized or unsupported Algorithm Identifier in timestamp."
        case .timestampBadRequest:
            "The timestamp transaction is not permitted or supported."
        case .timestampBadDataFormat:
            "The timestamp data submitted has the wrong format."
        case .timestampTimeNotAvailable:
            "The time source for the Timestamp Authority is not available."
        case .timestampUnacceptedPolicy:
            "The requested policy is not supported by the Timestamp Authority."
        case .timestampUnacceptedExtension:
            "The requested extension is not supported by the Timestamp Authority."
        case .timestampAddInfoNotAvailable:
            "The additional information requested is not available."
        case .timestampSystemFailure:
            "The timestamp request cannot be handled due to system failure."
        case .signingTimeMissing:
            "A signing time was expected but was not found."
        case .timestampRejection:
            "A timestamp transaction was rejected."
        case .timestampWaiting:
            "A timestamp transaction is waiting."
        case .timestampRevocationWarning:
            "A timestamp authority revocation warning was issued."
        case .timestampRevocationNotification:
            "A timestamp authority revocation notification was issued."
        case .unexpectedError:
            "Unexpected error has occurred."
        }
    }
}

extension Status: CustomNSError {
    var errorCode: Int {
        Int(rawValue)
    }

    var errorUserInfo: [String: Any] {
        [NSLocalizedDescriptionKey: description]
    }
}
