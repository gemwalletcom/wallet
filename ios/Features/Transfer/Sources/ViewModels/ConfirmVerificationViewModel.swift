// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Localization
import Primitives
import Style
import SwiftUI

struct ConfirmVerificationViewModel: ItemModelProvidable {
    private let infoAction: VoidAction

    init(infoAction: VoidAction) {
        self.infoAction = infoAction
    }

    var itemModel: ConfirmTransferItemModel {
        .verification(
            ListItemModel(
                title: Localized.Info.paymentVerificationTitle,
                subtitle: "",
                subtitleStyle: TextStyle(font: .body, color: Colors.orange),
                subtitleTagType: .image(Image(systemName: SystemImage.clockBadgeExclamationmark)),
                infoAction: infoAction,
            ),
        )
    }
}
