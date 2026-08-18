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

    public func balance(wallet _: Wallet, feeAsset _: Asset) async throws -> Balance {
        balance
    }
}
