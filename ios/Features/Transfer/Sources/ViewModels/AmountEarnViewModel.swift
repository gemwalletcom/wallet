// Copyright (c). Gem Wallet. All rights reserved.

import class Gemstone.GemAmountService
import BigInt
import Foundation
import enum Gemstone.GemAmountType
import protocol Gemstone.GemStakeServiceProtocol
import GemstonePrimitives
import Localization
import Primitives

public final class AmountEarnViewModel: AmountDataProvidable {
    let asset: Asset
    let action: EarnType
    private let stakeService: any GemStakeServiceProtocol
    private let wallet: Wallet
    let amountService: GemAmountService

    init(
        asset: Asset,
        action: EarnType,
        stakeService: any GemStakeServiceProtocol,
        wallet: Wallet,
        amountService: GemAmountService,
    ) {
        self.asset = asset
        self.action = action
        self.stakeService = stakeService
        self.wallet = wallet
        self.amountService = amountService
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
            recipient: Recipient(name: provider.name, address: provider.id, memo: nil),
            amount: nil,
        )
    }

    func makeTransferData(value: BigInt, useMaxAmount: Bool) async throws -> TransferData {
        let address = try wallet.account(for: asset.chain).address
        let earnData = try await ContractCallData(stakeService.getEarnData(
            assetId: asset.id.identifier,
            address: address,
            value: String(value),
            earnType: action.json(),
        ))
        return TransferData(
            type: .earn(asset, action, earnData),
            recipient: Recipient(name: provider.name, address: earnData.contractAddress, memo: nil),
            value: value,
            useMaxAmount: useMaxAmount,
        )
    }
}
