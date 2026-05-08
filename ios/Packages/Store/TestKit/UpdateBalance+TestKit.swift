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
    static func mockCoin(
        assetId: AssetId = .mock(),
        available: Double = 0,
        reserved: Double = 0,
        pendingUnconfirmed: Double = 0,
        updatedAt: Date = .now,
        isActive: Bool = true,
    ) -> Self {
        UpdateBalance(
            assetId: assetId,
            type: .coin(UpdateCoinBalance(
                available: .mock(amount: available),
                reserved: .mock(amount: reserved),
                pendingUnconfirmed: .mock(amount: pendingUnconfirmed),
            )),
            updatedAt: updatedAt,
            isActive: isActive,
        )
    }

    static func mockPerpetual(
        assetId: AssetId = Asset.hypercoreUSDC().id,
        available: Double = 0,
        reserved: Double = 0,
        withdrawable: Double = 0,
        updatedAt: Date = .now,
        isActive: Bool = true,
    ) -> Self {
        UpdateBalance(
            assetId: assetId,
            type: .perpetual(UpdatePerpetualBalance(
                available: .mock(amount: available),
                reserved: .mock(amount: reserved),
                withdrawable: .mock(amount: withdrawable),
            )),
            updatedAt: updatedAt,
            isActive: isActive,
        )
    }

    static func mockStake(
        assetId: AssetId = .mock(),
        staked: Double = 0,
        pending: Double = 0,
        frozen: Double = 0,
        locked: Double = 0,
        rewards: Double = 0,
        updatedAt: Date = .now,
        isActive: Bool = true,
    ) -> Self {
        UpdateBalance(
            assetId: assetId,
            type: .stake(UpdateStakeBalance(
                staked: .mock(amount: staked),
                pending: .mock(amount: pending),
                frozen: .mock(amount: frozen),
                locked: .mock(amount: locked),
                rewards: .mock(amount: rewards),
            )),
            updatedAt: updatedAt,
            isActive: isActive,
        )
    }

    static func mockEarn(
        assetId: AssetId = .mock(),
        balance: Double = 0,
        updatedAt: Date = .now,
        isActive: Bool = true,
    ) -> Self {
        UpdateBalance(
            assetId: assetId,
            type: .earn(UpdateEarnBalance(balance: .mock(amount: balance))),
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
                type: .token(UpdateTokenBalance(available: .mock(amount: Double(index)))),
                updatedAt: .now,
                isActive: true,
            )
        }
    }
}
