// Copyright (c). Gem Wallet. All rights reserved.

import protocol Gemstone.GemAmountServiceProtocol
import enum Gemstone.GemAmountType
import BigInt
import GemstonePrimitives
import Primitives
import struct Gemstone.GemTransferData

public enum AmountDataProvider: AmountDataProvidable, @unchecked Sendable {
    case transfer(AmountTransferViewModel)
    case stake(AmountStakeViewModel)
    case perpetual(AmountPerpetualViewModel)
    case earn(AmountEarnViewModel)

    static func make(from input: AmountInput, wallet: Wallet, service: any GemAmountServiceProtocol) -> AmountDataProvider {
        switch input.type {
        case let .transfer(recipient):
            .transfer(AmountTransferViewModel(asset: input.asset, action: .send(recipient)))
        case let .deposit(recipient):
            .transfer(AmountTransferViewModel(asset: input.asset, action: .deposit(recipient)))
        case let .withdraw(recipient):
            .transfer(AmountTransferViewModel(asset: input.asset, action: .withdraw(recipient)))
        case let .stake(stakeType):
            .stake(AmountStakeViewModel(asset: input.asset, type: stakeType))
        case let .perpetual(data):
            .perpetual(AmountPerpetualViewModel(asset: input.asset, data: data, service: service))
        case let .earn(earnType):
            .earn(AmountEarnViewModel(asset: input.asset, action: earnType, service: service, wallet: wallet))
        }
    }

    var asset: Asset {
        provider.asset
    }

    var title: String {
        provider.title
    }

    var amountType: AmountType {
        provider.amountType
    }

    var gemAmountType: GemAmountType {
        provider.gemAmountType
    }

    func recipientData() -> RecipientData {
        provider.recipientData()
    }

    func makeTransferData(value: BigInt, useMaxAmount: Bool) async throws -> GemTransferData {
        try await provider.makeTransferData(value: value, useMaxAmount: useMaxAmount)
    }
}

// MARK: - Private

extension AmountDataProvider {
    private var provider: any AmountDataProvidable {
        switch self {
        case let .transfer(provider): provider
        case let .stake(provider): provider
        case let .perpetual(provider): provider
        case let .earn(provider): provider
        }
    }
}
