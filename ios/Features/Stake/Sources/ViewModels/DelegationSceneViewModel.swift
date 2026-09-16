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
        switch service.delegationActionDestination(asset: asset.toGem(), delegation: model.delegation.toGem(), action: action, validators: validators.map { $0.toGem() }) {
        case .details: break
        case let .confirm(transfer): onTransferAction?(transfer)
        case let .amount(asset, input): onAmountInputAction?(AmountInput(type: input.map(), asset: asset.toPrimitives()))
        }
    }

    func onClaimRewards() {
        onTransferAction?(claimRewardsTransferData())
    }
}

// MARK: - Private

extension DelegationSceneViewModel {
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
