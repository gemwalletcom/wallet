// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Primitives
import PrimitivesTestKit
import Transfer

public struct FeeAssetProviderMock: FeeAssetProvidable {
    private let result: Result<AssetData, Error>

    public init(asset: Asset = .mock(), balance: Balance = .mock(), price: Price? = nil) {
        result = .success(.mock(asset: asset, balance: balance, price: price))
    }

    public init(error: String) {
        result = .failure(AnyError(error))
    }

    public func getAssetData(walletId _: WalletId, assetId _: AssetId) throws -> AssetData {
        try result.get()
    }
}
