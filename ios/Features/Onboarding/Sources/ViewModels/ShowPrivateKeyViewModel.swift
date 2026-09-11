// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Foundation
import GemstonePrimitives
import Localization
import Primitives
import PrimitivesComponents

struct ShowPrivateKeyViewModel: SecretPhraseViewableModel {
    let chain: Chain
    let text: String
    let continueAction: VoidAction = nil

    var calloutViewStyle: CalloutViewStyle? {
        .privateKeyWarning(chainName: chain.networkName)
    }

    var title: String {
        Localized.Common.privateKey
    }

    var copyModel: CopyTypeViewModel {
        CopyTypeViewModel(
            type: .privateKey,
            copyValue: text,
        )
    }

    var type: SecretPhraseDataType {
        .privateKey(key: text)
    }
}
