// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import Foundation
import enum Gemstone.GemAmountType
import protocol Gemstone.GemAmountServiceProtocol
import struct Gemstone.GemValidatorRow
import enum Gemstone.EarnType
import GemstonePrimitives
import Localization
import Primitives
import struct Gemstone.GemTransferData

public final class AmountEarnViewModel: AmountDataProvidable {
    let asset: Asset
    let action: Gemstone.EarnType
    private let service: any GemAmountServiceProtocol

    init(asset: Asset, action: Gemstone.EarnType, service: any GemAmountServiceProtocol) {
        self.asset = asset
        self.action = action
        self.service = service
    }

    var provider: DelegationValidator {
        switch action {
        case let .deposit(provider): provider.map()
        case let .withdraw(delegation): delegation.validator.map()
        }
    }

    var providerRow: GemValidatorRow {
        service.validatorRow(validator: provider.map())
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

    var gemAmountType: GemAmountType {
        service.earnAmountType(earnType: action)
    }

    func makeTransferData(value: BigInt, useMaxAmount: Bool) async throws -> GemTransferData {
        try await service.earnTransferData(asset: asset.map(), earnType: action, value: value, useMaxAmount: useMaxAmount)
    }
}
