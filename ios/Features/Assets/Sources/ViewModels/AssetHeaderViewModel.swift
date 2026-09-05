// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Foundation
import struct Gemstone.GemAssetDetailsState
import Primitives
import PrimitivesComponents
import Style
import SwiftUI

struct AssetHeaderViewModel {
    let assetDataModel: AssetDataViewModel
    let state: GemAssetDetailsState
}

extension AssetHeaderViewModel: ValueHeaderViewModel {
    var isWatchWallet: Bool {
        state.isViewOnly
    }

    var assetImage: AssetImage? {
        assetDataModel.assetImage
    }

    var title: String {
        assetDataModel.totalBalanceTextWithSymbol
    }

    var subtitle: String? {
        if assetDataModel.fiatBalanceText.isEmpty {
            return .none
        }
        return assetDataModel.fiatBalanceText
    }

    var subtitleColor: Color {
        Colors.gray
    }

    var buttons: [HeaderButton] {
        state.headerButtons.map { HeaderButton(type: $0.kind.headerButtonType, isEnabled: $0.isEnabled) }
    }
}
