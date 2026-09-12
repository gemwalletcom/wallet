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
        AssetViewModel(asset: data.asset.map()).assetImage
    }

    var title: String {
        switch data.value {
        case .unlimited:
            Localized.Simulation.Header.unlimitedAsset(data.asset.symbol)
        case let .exact(value):
            Self.formatter.string(
                BigInt(value),
                decimals: Int(data.asset.decimals),
                currency: data.asset.symbol,
            )
        }
    }

    var subtitle: String? {
        nil
    }

    var subtitleColor: Color {
        Colors.gray
    }
}
