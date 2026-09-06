// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import Foundation
import enum Gemstone.GemAmountType
import protocol Gemstone.GemAmountServiceProtocol
import GemstonePrimitives
import Localization
import Primitives
import struct Gemstone.GemTransferData

public final class AmountEarnViewModel: AmountDataProvidable {
    let asset: Asset
    let action: EarnType
    private let service: any GemAmountServiceProtocol

    init(asset: Asset, action: EarnType, service: any GemAmountServiceProtocol) {
        self.asset = asset
        self.action = action
        self.service = service
    }

    var provider: DelegationValidator {
        switch action {
        case let .deposit(provider): provider
        case let .withdraw(delegation): delegation.validator
        }
    }

    var providerTitle: String {
        Localized.Common.provider
    }

    var title: String {
        switch action {
        case .deposit: Localized.Wallet.deposit
        case .withdraw: Localized.Wallet.withdraw
        }
    }

    var amountType: AmountType {
        .earn(action)
    }

    var gemAmountType: GemAmountType {
        service.earnAmountType(earnType: action.map())
    }

    func makeTransferData(value: BigInt, useMaxAmount: Bool) async throws -> GemTransferData {
        try await service.earnTransferData(asset: asset.map(), earnType: action.map(), value: value, useMaxAmount: useMaxAmount)
    }
}
