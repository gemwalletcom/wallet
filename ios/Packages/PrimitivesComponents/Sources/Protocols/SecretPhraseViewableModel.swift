// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Foundation
import GemstonePrimitives
import Primitives

public protocol SecretPhraseViewableModel {
    var title: String { get }
    var calloutViewStyle: CalloutViewStyle? { get }
    var type: SecretPhraseDataType { get }
    var copyModel: CopyTypeViewModel { get }
    var continueAction: VoidAction { get }
    var docsUrl: URL { get }
}

public extension SecretPhraseViewableModel {
    var docsUrl: URL {
        AppUrl.docs(.howToSecureSecretPhrase)
    }
}
