// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import Components
import Primitives
import PrimitivesComponents
import Style
import SwiftUI

public struct ConfirmBalanceChangeViewModel {
    private let balanceChange: SimulationAssetChange

    init(balanceChange: SimulationAssetChange) {
        self.balanceChange = balanceChange
    }

    public var assetTitle: String {
        balanceChange.asset.name
    }

    public var assetImage: AssetImage {
        AssetIdViewModel(assetId: balanceChange.asset.id).assetImage
    }

    public var amount: TextValue {
        NumericViewModel(
            data: AssetValuePrice(asset: balanceChange.asset, value: abs(balanceChange.value), price: nil),
            style: AmountDisplayStyle(
                sign: amountSign,
                formatter: .full,
                currencyCode: "",
                textStyle: TextStyle(font: .body, color: amountColor, fontWeight: .medium),
            ),
        ).amount
    }

    private var amountSign: AmountDisplaySign {
        if balanceChange.value > BigInt.zero {
            .incoming
        } else if balanceChange.value < BigInt.zero {
            .outgoing
        } else {
            .none
        }
    }

    private var amountColor: Color {
        PriceChangeColor.color(for: Double(balanceChange.value.signum()))
    }
}
