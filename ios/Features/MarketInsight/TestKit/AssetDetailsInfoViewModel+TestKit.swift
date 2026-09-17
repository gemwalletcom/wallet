// Copyright (c). Gem Wallet. All rights reserved.

@testable import MarketInsight
import Primitives
import PrimitivesTestKit

extension AssetDetailsInfoViewModel {
    static func mock() -> AssetDetailsInfoViewModel {
        AssetDetailsInfoViewModel(asset: .mockEthereumUSDT(), currency: .usd)
    }
}
