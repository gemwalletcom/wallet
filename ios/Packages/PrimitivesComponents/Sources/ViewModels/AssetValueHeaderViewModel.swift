// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import Components
import Formatters
import struct Gemstone.GemSimulationValue
import GemstonePrimitives
import Foundation
import Localization
import Primitives
import Style
import SwiftUI

struct AssetValueHeaderViewModel: ValueHeaderViewModel {
    private static let formatter = ValueFormatter(style: .full)

    let data: GemSimulationValue

    let isWatchWallet: Bool = false
    let buttons: [HeaderButton] = []

    var assetImage: AssetImage? {
        AssetViewModel(asset: data.asset.toPrimitives()).assetImage
    }

    var title: String {
        data.value.title(symbol: data.asset.symbol, formatter: Self.formatter, decimals: Int(data.asset.decimals))
    }

    var subtitle: String? {
        nil
    }

    var subtitleColor: Color {
        Colors.gray
    }
}
