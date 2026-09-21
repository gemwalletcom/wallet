// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import Components
import Formatters
import Foundation
import struct Gemstone.GemSimulationValue
import GemstonePrimitives
import Localization
import Primitives
import Style
import SwiftUI

public struct AssetValueHeaderViewModel: ValueHeaderViewModel {
    private static let formatter = ValueFormatter(style: .full)

    public let data: GemSimulationValue

    public init(data: GemSimulationValue) {
        self.data = data
    }

    public let isWatchWallet: Bool = false
    public let buttons: [HeaderButton] = []

    public var assetImage: AssetImage? {
        AssetViewModel(asset: data.asset.toPrimitives()).assetImage
    }

    public var title: String {
        data.value.title(symbol: data.asset.symbol, formatter: Self.formatter, decimals: Int(data.asset.decimals))
    }

    public var subtitle: String? {
        nil
    }

    public var subtitleColor: Color {
        Colors.gray
    }
}
