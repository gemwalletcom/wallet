// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Primitives
import PrimitivesTestKit
import Transfer

public struct FeeAssetProviderMock: FeeAssetProvidable {
    private let asset: Asset
    private let balance: Balance

    public init(asset: Asset = .mock(), balance: Balance = .mock()) {
        self.asset = asset
        self.balance = balance
    }

    public func load(wallet _: Wallet, feeAssetId _: AssetId) async throws -> (Asset, Balance) {
        (asset, balance)
    }
}
