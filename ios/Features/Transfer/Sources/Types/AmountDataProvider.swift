// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import Primitives

public enum AmountDataProvider: AmountDataProvidable, @unchecked Sendable {
    case transfer(AmountTransferViewModel)
    case stake(AmountStakeViewModel)
    case freeze(AmountFreezeViewModel)
    case perpetual(AmountPerpetualViewModel)
    case earn(AmountEarnViewModel)

    static func make(
        from input: AmountInput,
        wallet: Wallet,
        service: AmountService,
    ) -> AmountDataProvider {
        switch input.type {
        case let .transfer(recipient):
            .transfer(AmountTransferViewModel(asset: input.asset, action: .send(recipient)))
        case let .deposit(recipient):
            .transfer(AmountTransferViewModel(asset: input.asset, action: .deposit(recipient)))
        case let .withdraw(recipient):
            .transfer(AmountTransferViewModel(asset: input.asset, action: .withdraw(recipient)))
        case let .stake(stakeType):
            .stake(AmountStakeViewModel(asset: input.asset, action: stakeType))
        case let .freeze(resource):
            .freeze(AmountFreezeViewModel(asset: input.asset, action: .freeze, resource: resource))
        case let .unfreeze(resource):
            .freeze(AmountFreezeViewModel(asset: input.asset, action: .unfreeze, resource: resource))
        case let .perpetual(data):
            .perpetual(AmountPerpetualViewModel(asset: input.asset, data: data))
        case let .earn(earnType):
            .earn(AmountEarnViewModel(asset: input.asset, action: earnType, earnService: service.earnDataProvider, wallet: wallet))
        }
    }

    var asset: Asset { provider.asset }
    var title: String { provider.title }
    var amountType: AmountType { provider.amountType }
    var minimumValue: BigInt { provider.minimumValue }
    var canChangeValue: Bool { provider.canChangeValue }
    var reserveForFee: BigInt { provider.reserveForFee }

    func availableValue(from assetData: AssetData) -> BigInt {
        provider.availableValue(from: assetData)
    }

    func shouldReserveFee(from assetData: AssetData) -> Bool {
        provider.shouldReserveFee(from: assetData)
    }

    func maxValue(from assetData: AssetData) -> BigInt {
        provider.maxValue(from: assetData)
    }

    func recipientData() -> RecipientData {
        provider.recipientData()
    }

    func makeTransferData(value: BigInt) async throws -> TransferData {
        try await provider.makeTransferData(value: value)
    }
}

// MARK: - Private

extension AmountDataProvider {
    private var provider: any AmountDataProvidable {
        switch self {
        case let .transfer(provider): provider
        case let .stake(provider): provider
        case let .freeze(provider): provider
        case let .perpetual(provider): provider
        case let .earn(provider): provider
        }
    }
}
