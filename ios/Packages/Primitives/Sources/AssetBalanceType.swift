// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import Foundation

public struct AssetBalance: Codable, Sendable {
    public let assetId: AssetId
    public let balance: Balance
    public let isActive: Bool

    public init(
        assetId: AssetId,
        balance: Balance,
        isActive: Bool = true,
    ) {
        self.assetId = assetId
        self.balance = balance
        self.isActive = isActive
    }
}

public extension AssetBalance {
    static func merge(assetIds: [AssetId], balances: [BigInt]) -> [AssetBalance] {
        zip(assetIds, balances).map {
            AssetBalance(assetId: $0, balance: Balance(available: $1))
        }
    }
}

public struct WalletAssetBalance: Codable, Sendable {
    public let walletId: String
    public let balance: AssetBalance

    public init(
        walletId: String,
        balance: AssetBalance,
    ) {
        self.walletId = walletId
        self.balance = balance
    }
}

public extension AssetBalance {
    static func make(
        for assetId: AssetId,
        balance: Balance = Balance(available: .zero),
    ) -> AssetBalance {
        AssetBalance(
            assetId: assetId,
            balance: balance,
        )
    }
}
