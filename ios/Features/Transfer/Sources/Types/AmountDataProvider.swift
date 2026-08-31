import protocol Gemstone.GemPreferencesServiceProtocol
// Copyright (c). Gem Wallet. All rights reserved.

import class Gemstone.GemAmountService
import enum Gemstone.GemAmountType
import BigInt
import Primitives

public enum AmountDataProvider: AmountDataProvidable, @unchecked Sendable {
    case transfer(AmountTransferViewModel)
    case stake(AmountStakeViewModel)
    case perpetual(AmountPerpetualViewModel)
    case earn(AmountEarnViewModel)

    static func make(
        from input: AmountInput,
        wallet: Wallet,
        service: AmountService,
        preferencesService: any GemPreferencesServiceProtocol,
    ) -> AmountDataProvider {
        switch input.type {
        case let .transfer(recipient):
            .transfer(AmountTransferViewModel(asset: input.asset, action: .send(recipient), amountService: service.amountService))
        case let .deposit(recipient):
            .transfer(AmountTransferViewModel(asset: input.asset, action: .deposit(recipient), amountService: service.amountService))
        case let .withdraw(recipient):
            .transfer(AmountTransferViewModel(asset: input.asset, action: .withdraw(recipient), amountService: service.amountService))
        case let .stake(stakeType):
            .stake(AmountStakeViewModel(asset: input.asset, type: stakeType, amountService: service.amountService))
        case let .perpetual(data):
            .perpetual(AmountPerpetualViewModel(asset: input.asset, data: data, preferencesService: preferencesService, amountService: service.amountService))
        case let .earn(earnType):
            .earn(AmountEarnViewModel(asset: input.asset, action: earnType, stakeService: service.stakeService, wallet: wallet, amountService: service.amountService))
        }
    }

    var asset: Asset {
        provider.asset
    }

    var amountService: GemAmountService {
        provider.amountService
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

    func makeTransferData(value: BigInt, useMaxAmount: Bool) async throws -> TransferData {
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
