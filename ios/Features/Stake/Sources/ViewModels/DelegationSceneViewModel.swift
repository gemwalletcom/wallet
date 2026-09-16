// Copyright (c). Gem Wallet. All rights reserved.

import protocol Gemstone.GemStakeServiceProtocol
import Components
import Foundation
import enum Gemstone.GemDelegationAction
import enum Gemstone.GemDelegationCompletion
import enum Gemstone.GemDelegationRow
import GemstonePrimitives
import Localization
import Primitives
import PrimitivesComponents
import Style
import SwiftUI
import struct Gemstone.GemTransferData

public struct DelegationSceneViewModel {
    public let model: DelegationViewModel
    public let validators: [DelegationValidator]
    public let onAmountInputAction: AmountInputAction
    public let onTransferAction: TransferDataAction

    private let wallet: Wallet
    private let asset: Asset
    private let service: any GemStakeServiceProtocol

    public init(
        wallet: Wallet,
        model: DelegationViewModel,
        asset: Asset,
        service: any GemStakeServiceProtocol,
        validators: [DelegationValidator],
        onAmountInputAction: AmountInputAction,
        onTransferAction: TransferDataAction,
    ) {
        self.wallet = wallet
        self.model = model
        self.asset = asset
        self.service = service
        self.validators = validators
        self.onAmountInputAction = onAmountInputAction
        self.onTransferAction = onTransferAction
    }

    public var title: String {
        providerType.title
    }

    public var rows: [GemDelegationRow] {
        service.delegationRows(delegation: model.delegation.toGem())
    }

    public func title(for row: GemDelegationRow) -> String {
        delegationRowTitle(row, providerType: providerType, completion: model.status.completion)
    }

    public var aprModel: AprViewModel {
        AprViewModel(apr: model.delegation.validator.apr)
    }

    public var manageTitle: String {
        Localized.Common.manage
    }

    public var stateModel: DelegationStateViewModel {
        model.stateModel
    }

    public var providerUrl: URL? {
        switch providerType {
        case .stake: model.validatorUrl
        case .earn: nil
        }
    }

    public var assetImageStyle: ListItemImageStyle? {
        .asset(assetImage: AssetViewModel(asset: asset).assetImage)
    }

    public var availableActions: [GemDelegationAction] {
        service.delegationActions(walletType: wallet.type.toGem(), delegation: model.delegation.toGem())
    }

    public var showManage: Bool {
        availableActions.isNotEmpty
    }

    public var canClaimRewards: Bool {
        service.canClaimDelegationRewards(walletType: wallet.type.toGem(), delegation: model.delegation.toGem())
    }

}

// MARK: - Actions

public extension DelegationSceneViewModel {
    func onSelectAction(_ action: GemDelegationAction) {
        switch action {
        case .stake:
            onAmountInputAction?(amountInput(.stake(.stake(validators: validators.map { $0.toGem() }, validator: model.delegation.validator.toGem()))))
        case .unstake:
            if service.canChangeAmountOnUnstake(chain: asset.chain.rawValue) {
                onAmountInputAction?(amountInput(.stake(.unstake(delegation: model.delegation.toGem()))))
            } else {
                onTransferAction?(stakeTransferData(.unstake(model.delegation)))
            }
        case .redelegate:
            onAmountInputAction?(amountInput(.stake(.redelegate(validators: validators.map { $0.toGem() }, delegation: model.delegation.toGem(), validator: nil))))
        case .deposit:
            onAmountInputAction?(amountInput(.earn(.deposit(model.delegation.validator.toGem()))))
        case .withdraw:
            switch providerType {
            case .stake: onTransferAction?(stakeTransferData(.withdraw(model.delegation)))
            case .earn: onAmountInputAction?(amountInput(.earn(.withdraw(model.delegation.toGem()))))
            }
        }
    }

    func onClaimRewards() {
        onTransferAction?(claimRewardsTransferData())
    }
}

// MARK: - Private

extension DelegationSceneViewModel {
    private func amountInput(_ type: AmountType) -> AmountInput {
        AmountInput(type: type, asset: asset)
    }

    private func stakeTransferData(_ stakeType: StakeType) -> GemTransferData {
        service.stakeTransferData(asset: asset.toGem(), stakeType: stakeType.toGem(), value: model.delegation.base.balance, useMaxAmount: false)
    }

    private func claimRewardsTransferData() -> GemTransferData {
        service.stakeTransferData(
            asset: asset.toGem(),
            stakeType: StakeType.rewards([model.delegation.validator]).toGem(),
            value: model.delegation.base.rewards,
            useMaxAmount: false,
        )
    }

    private var providerType: StakeProviderType {
        model.delegation.validator.providerType
    }

}

extension GemDelegationAction: @retroactive Identifiable {
    public var id: Self { self }
}
