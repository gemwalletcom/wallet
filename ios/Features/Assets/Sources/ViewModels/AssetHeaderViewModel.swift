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
    let assetId: AssetId
    let details: GemAssetDetails
}

extension AssetHeaderViewModel: ValueHeaderViewModel {
    var isWatchWallet: Bool {
        details.state.headerActions.isWatchOnly
    }

    var assetImage: AssetImage? {
        AssetIdViewModel(assetId: assetId).assetImage
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
        details.state.headerActions.headerButtons
    }
}
