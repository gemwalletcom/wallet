// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Foundation
import struct Gemstone.GemAssetDetails
import GemstonePrimitives
import Primitives
import PrimitivesComponents
import Style
import SwiftUI

struct AssetHeaderViewModel {
    let assetDataModel: AssetDataViewModel
    let details: GemAssetDetails
}

extension AssetHeaderViewModel: ValueHeaderViewModel {
    var isWatchWallet: Bool {
        details.state.headerActions == .watchOnly
    }

    var assetImage: AssetImage? {
        assetDataModel.assetImage
    }

    var title: String {
        details.balanceValue.text()
    }

    var subtitle: String? {
        details.fiatValue?.text()
    }

    var subtitleColor: Color {
        Colors.gray
    }

    var buttons: [HeaderButton] {
        switch details.state.headerActions {
        case .watchOnly: []
        case let .buttons(buttons): buttons.map { HeaderButton(type: $0.kind, isEnabled: $0.isEnabled) }
        }
    }
}
