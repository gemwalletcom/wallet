// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import Foundation
import protocol Gemstone.GemStakeServiceProtocol
import GemstonePrimitives
import Localization
import Primitives

public final class AmountEarnViewModel: AmountDataProvidable {
    let asset: Asset
    let action: EarnType
    private let stakeService: any GemStakeServiceProtocol
    private let wallet: Wallet

    init(
        asset: Asset,
        action: EarnType,
        stakeService: any GemStakeServiceProtocol,
        wallet: Wallet,
    ) {
        self.asset = asset
        self.action = action
        self.stakeService = stakeService
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

    var minimumValue: BigInt {
        .zero
    }

    var canChangeValue: Bool {
        true
    }

    var reserveForFee: BigInt {
        .zero
    }

    func shouldReserveFee(from _: AssetData) -> Bool {
        false
    }

    func availableValue(from assetData: AssetData) -> BigInt {
        switch action {
        case .deposit: assetData.balance.available
        case let .withdraw(delegation): delegation.base.balanceValue
        }
    }

    func maxValue(from assetData: AssetData) -> BigInt {
        availableValue(from: assetData)
    }

    func recipientData() -> RecipientData {
        RecipientData(
            recipient: Recipient(name: provider.name, address: provider.id, memo: nil),
            amount: nil,
        )
    }

    func makeTransferData(amount: TransferAmountValue) async throws -> TransferData {
        let address = try wallet.account(for: asset.chain).address
        let earnData = try await ContractCallData(stakeService.getEarnData(
            assetId: asset.id.identifier,
            address: address,
            value: String(amount.value),
            earnType: action.json(),
        ))
        return TransferData(
            type: .earn(asset, action, earnData),
            recipientData: RecipientData(
                recipient: Recipient(name: provider.name, address: earnData.contractAddress, memo: nil),
                amount: nil,
            ),
            amount: amount,
        )
    }
}
