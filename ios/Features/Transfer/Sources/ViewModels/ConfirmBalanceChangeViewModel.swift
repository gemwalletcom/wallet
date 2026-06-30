// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import Components
import ExplorerService
import Localization
import Primitives
import PrimitivesComponents
import Style
import SwiftUI

public struct ConfirmBalanceChangeViewModel {
    private let balanceChange: SimulationAssetChange

    init(balanceChange: SimulationAssetChange) {
        self.balanceChange = balanceChange
    }

    var isUnknown: Bool {
        balanceChange.name == nil
    }

    public var assetTitle: String {
        balanceChange.name ?? balanceChange.symbol ?? Localized.Errors.unknown
    }

    public var assetImage: AssetImage {
        AssetIdViewModel(assetId: balanceChange.assetId).assetImage
    }

    public var amount: TextValue {
        NumericViewModel(
            data: AssetValuePrice(asset: asset, value: abs(balanceChange.value), price: nil),
            style: AmountDisplayStyle(
                sign: amountSign,
                currencyCode: "",
                textStyle: TextStyle(font: .body, color: amountColor, fontWeight: .medium),
            ),
        ).amount
    }

    var explorerTokenURL: URL? {
        guard let tokenId = balanceChange.assetId.tokenId else {
            return nil
        }
        return ExplorerService.standard.tokenUrl(chain: balanceChange.assetId.chain, address: tokenId)?.url
    }

    private var asset: Asset {
        Asset(
            id: balanceChange.assetId,
            name: balanceChange.name ?? "",
            symbol: balanceChange.symbol ?? "",
            decimals: balanceChange.decimals,
            type: balanceChange.assetId.tokenId == nil ? .native : .token,
        )
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
