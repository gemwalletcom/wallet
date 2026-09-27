// Copyright (c). Gem Wallet. All rights reserved.

import Components
import struct Gemstone.GemNodeRow
import Localization
import PrimitivesComponents

extension GemNodeRow {
    var listItem: ListItemModel {
        latencyStatus.listItem(
            title: title.text(gemNodeLabel: Localized.Nodes.gemWalletNode),
            titleExtra: subtitle.text,
        )
    }
}
