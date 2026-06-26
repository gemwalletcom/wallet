// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import Components
import Formatters
import PrimitivesComponents
import Style
import SwiftUI

public struct ConfirmBalanceChangeViewModel {
    private static let formatter = ValueFormatter(style: .full)

    private let balanceChange: SimulationAssetChange

    init(balanceChange: SimulationAssetChange) {
        self.balanceChange = balanceChange
    }

    private var assetViewModel: AssetViewModel {
        AssetViewModel(asset: balanceChange.asset)
    }

    public var assetTitle: String {
        assetViewModel.title
    }

    public var assetImage: AssetImage {
        assetViewModel.assetImage
    }

    public var amount: String {
        let value = balanceChange.value < BigInt.zero ? -balanceChange.value : balanceChange.value
        let formatted = Self.formatter.string(value, asset: balanceChange.asset)
        if balanceChange.value > BigInt.zero {
            return "+\(formatted)"
        }
        if balanceChange.value < BigInt.zero {
            return "-\(formatted)"
        }
        return formatted
    }

    public var color: Color {
        PriceChangeColor.color(for: Double(balanceChange.value.signum()))
    }
}
