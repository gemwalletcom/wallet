// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import Foundation
import enum Gemstone.GemAmountType
import protocol Gemstone.GemAmountServiceProtocol
import GemstonePrimitives
import Localization
import Primitives
import struct Gemstone.GemRecipient
import struct Gemstone.GemTransferData

public final class AmountEarnViewModel: AmountDataProvidable {
    let asset: Asset
    let action: EarnType
    private let service: any GemAmountServiceProtocol
    private let wallet: Wallet

    init(asset: Asset, action: EarnType, service: any GemAmountServiceProtocol, wallet: Wallet) {
        self.asset = asset
        self.action = action
        self.service = service
        self.wallet = wallet
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
        switch action {
        case .deposit: .earn(earnType: .deposit)
        case let .withdraw(delegation): .earn(earnType: .withdraw(delegation: delegation.json()))
        }
    }

    func recipientData() -> RecipientData {
        RecipientData(
            recipient: GemRecipient(address: provider.id, name: provider.name),
            amount: nil,
        )
    }

    func makeTransferData(value: BigInt, useMaxAmount: Bool) async throws -> GemTransferData {
        let address = try wallet.account(for: asset.chain).address
        let earnData = try await ContractCallData(service.earnData(
            assetId: asset.id.identifier,
            address: address,
            value: String(value),
            earnType: action.json(),
        ))
        return GemTransferData(
            inputType: .earn(asset, action, earnData),
            recipient: GemRecipient(address: earnData.contractAddress, name: provider.name, memo: nil),
            value: value,
            useMaxAmount: useMaxAmount,
        )
    }
}
