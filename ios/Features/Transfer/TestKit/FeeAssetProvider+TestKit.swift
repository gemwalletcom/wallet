// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Primitives
import PrimitivesTestKit
import Transfer

public struct FeeAssetProviderMock: FeeAssetProvidable {
    private let result: Result<AssetData, Error>
    private let feeAssetsResult: Result<[AssetData], Error>

    public init(
        asset: Asset = .mock(),
        balance: Balance = .mock(),
        price: Price? = nil,
    ) {
        result = .success(.mock(asset: asset, balance: balance, price: price))
        feeAssetsResult = .success([])
    }

    public init(error: String) {
        let error = AnyError(error)
        result = .failure(error)
        feeAssetsResult = .failure(error)
    }

    public func feeAssets(walletId _: WalletId, chain _: Chain) async throws -> [AssetData] {
        try feeAssetsResult.get()
    }

    public func getAssetData(walletId _: WalletId, assetId _: AssetId) throws -> AssetData {
        try result.get()
    }
}
