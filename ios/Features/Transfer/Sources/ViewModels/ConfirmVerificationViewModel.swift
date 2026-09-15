// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Localization
import Primitives

struct ConfirmVerificationViewModel: ItemModelProvidable {
    private let infoAction: VoidAction

    init(infoAction: VoidAction) {
        self.infoAction = infoAction
    }

    var itemModel: ConfirmTransferItemModel {
        .verification(ListItemModel(title: Localized.Info.paymentVerificationTitle, infoAction: infoAction))
    }
}
