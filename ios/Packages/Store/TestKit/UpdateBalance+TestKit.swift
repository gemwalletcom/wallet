// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Primitives
import PrimitivesTestKit
import Store

private extension UpdateBalanceValue {
    static func mock(amount: Double = 0) -> Self {
        let value = if amount.rounded(.towardZero) == amount {
            String(Int64(amount))
        } else {
            "\(amount)"
        }
        return UpdateBalanceValue(value: value, amount: amount)
    }
}

public extension UpdateBalance {
    static func mock(
        assetId: AssetId = .mock(),
        available: Double = 0,
        reserved: Double = 0,
        withdrawable: Double = 0,
        pendingUnconfirmed: Double = 0,
        updatedAt: Date = .now,
        isActive: Bool = true,
    ) -> Self {
        UpdateBalance(
            assetId: assetId,
            available: .mock(amount: available),
            pendingUnconfirmed: .mock(amount: pendingUnconfirmed),
            reserved: .mock(amount: reserved),
            withdrawable: .mock(amount: withdrawable),
            updatedAt: updatedAt,
            isActive: isActive,
        )
    }
}

public extension [UpdateBalance] {
    static func mock(assets: [AssetBasic] = .mock()) -> Self {
        assets.enumerated().compactMap { index, asset in
            // skip the first asset to avoid having all mocks with a balance
            guard index > 0 else { return nil }
            return UpdateBalance(
                assetId: asset.asset.id,
                available: .mock(amount: Double(index)),
                updatedAt: .now,
                isActive: true,
            )
        }
    }
}
