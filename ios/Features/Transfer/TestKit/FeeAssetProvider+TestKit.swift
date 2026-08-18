// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Primitives
import PrimitivesTestKit
import Transfer

public struct FeeAssetProviderMock: FeeAssetProvidable {
    private let balance: Balance

    public init(balance: Balance = .mock()) {
        self.balance = balance
    }

    public func feeAsset(wallet _: Wallet, asset: Asset, fee _: Fee) async throws -> (asset: Asset, balance: Balance) {
        (asset, balance)
    }
}
