// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Primitives
import PrimitivesTestKit
import Transfer

public struct FeeAssetProviderMock: FeeAssetProvidable {
    private enum Result {
        case success(AssetData)
        case failure(String)
    }

    private let result: Result

    public init(asset: Asset = .mock(), balance: Balance = .mock(), price: Price? = nil) {
        result = .success(.mock(asset: asset, balance: balance, price: price))
    }

    public init(error: String) {
        result = .failure(error)
    }

    public func load(walletId _: WalletId, feeAssetId _: AssetId) throws -> AssetData {
        switch result {
        case let .success(assetData): assetData
        case let .failure(error): throw AnyError(error)
        }
    }
}
