// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Foundation
import Localization
import Primitives
import PrimitivesComponents
import func Gemstone.privateKeyCopy

struct ShowPrivateKeyViewModel: SecretPhraseViewableModel {
    let text: String
    let continueAction: VoidAction = nil

    var calloutViewStyle: CalloutViewStyle? {
        .secretDataWarning()
    }

    var title: String {
        Localized.Common.privateKey
    }

    var copyModel: CopyTypeViewModel {
        CopyTypeViewModel(content: privateKeyCopy(key: text))
    }

    var type: SecretPhraseDataType {
        .privateKey(key: text)
    }
}
