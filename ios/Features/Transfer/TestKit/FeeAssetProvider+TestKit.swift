// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Primitives
import PrimitivesTestKit
import Transfer

public struct FeeAssetProviderMock: FeeAssetProvidable {
    private let assetData: AssetData

    public init(asset: Asset = .mock(), balance: Balance = .mock(), price: Price? = nil) {
        assetData = .mock(asset: asset, balance: balance, price: price)
    }

    public func load(walletId _: WalletId, feeAssetId _: AssetId) throws -> AssetData {
        assetData
    }
}
