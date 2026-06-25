// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import Components
import Formatters
import PrimitivesComponents
import Style
import SwiftUI

struct ConfirmBalanceChangeViewModel {
    private static let formatter = ValueFormatter(style: .full)

    private let balanceChange: SimulationAssetChange

    init(balanceChange: SimulationAssetChange) {
        self.balanceChange = balanceChange
    }

    private var assetViewModel: AssetViewModel {
        AssetViewModel(asset: balanceChange.asset)
    }

    var name: String {
        assetViewModel.name
    }

    var assetImage: AssetImage {
        assetViewModel.assetImage
    }

    var title: String {
        let amount = balanceChange.value < BigInt.zero ? -balanceChange.value : balanceChange.value
        let value = Self.formatter.string(amount, asset: balanceChange.asset)
        if balanceChange.value > BigInt.zero {
            return "+\(value)"
        }
        if balanceChange.value < BigInt.zero {
            return "-\(value)"
        }
        return value
    }

    var color: Color {
        PriceChangeColor.color(for: Double(balanceChange.value.signum()))
    }

    var amountTextValue: TextValue {
        TextValue(text: title, style: TextStyle(font: .callout, color: color, fontWeight: .medium))
    }
}
