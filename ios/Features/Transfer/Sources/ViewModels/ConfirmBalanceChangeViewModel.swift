// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import Components
import ExplorerService
import Formatters
import Localization
import Primitives
import PrimitivesComponents
import Style
import SwiftUI

public struct ConfirmBalanceChangeViewModel {
    private static let formatter = ValueFormatter(style: .full)

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

    public var amount: String {
        let value = balanceChange.value < BigInt.zero ? -balanceChange.value : balanceChange.value
        let formatted = Self.formatter.string(value, decimals: Int(balanceChange.decimals), currency: balanceChange.symbol ?? "")
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

    var explorerTokenURL: URL? {
        guard let tokenId = balanceChange.assetId.tokenId else {
            return nil
        }
        return ExplorerService.standard.tokenUrl(chain: balanceChange.assetId.chain, address: tokenId)?.url
    }
}
