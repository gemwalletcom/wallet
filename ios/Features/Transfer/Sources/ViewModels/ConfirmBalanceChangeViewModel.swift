// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import Components
import struct Gemstone.GemSimulationBalanceChange
import GemstonePrimitives
import Primitives
import PrimitivesComponents
import Style
import SwiftUI

public struct ConfirmBalanceChangeViewModel {
    private let balanceChange: GemSimulationBalanceChange
    private let asset: Asset

    init(balanceChange: GemSimulationBalanceChange) {
        self.balanceChange = balanceChange
        asset = balanceChange.asset.map()
    }

    public var assetTitle: String {
        asset.name
    }

    public var assetImage: AssetImage {
        AssetIdViewModel(assetId: asset.id).assetImage
    }

    public var amount: TextValue {
        NumericViewModel(
            data: AssetValuePrice(asset: asset, value: abs(balanceChange.value), price: nil),
            style: AmountDisplayStyle(
                sign: balanceChange.sign,
                formatter: .full,
                currencyCode: "",
                textStyle: TextStyle(font: .body, color: amountColor, fontWeight: .medium),
            ),
        ).amount
    }

    private var amountColor: Color {
        switch balanceChange.sign {
        case .incoming: Colors.green
        case .outgoing: Colors.red
        case .none: Colors.gray
        }
    }
}
