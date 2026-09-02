// Copyright (c). Gem Wallet. All rights reserved.

import protocol Gemstone.GemStakeServiceProtocol
import Components
import Foundation
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
        switch providerType {
        case .stake: Localized.Transfer.Stake.title
        case .earn: Localized.Common.earn
        }
    }

    public var providerField: ListItemField {
        let title: String = switch providerType {
        case .stake: Localized.Stake.validator
        case .earn: Localized.Common.provider
        }
        return ListItemField(title: title, value: model.validatorText)
    }

    public var aprModel: AprViewModel {
        AprViewModel(apr: model.delegation.validator.apr)
    }

    public var stateTitle: String {
        Localized.Transaction.status
    }

    public var manageTitle: String {
        Localized.Common.manage
    }

    public var rewardsTitle: String {
        Localized.Stake.rewards
    }

    public var stateModel: DelegationStateViewModel {
        DelegationStateViewModel(state: model.state)
    }

    public var providerUrl: URL? {
        switch providerType {
        case .stake: model.validatorUrl
        case .earn: nil
        }
    }

    public var completionDateField: ListItemField? {
        let title: String? = switch providerType {
        case .stake:
            switch model.state {
            case .pending, .deactivating: Localized.Stake.availableIn
            case .activating: Localized.Stake.activeIn
            default: .none
            }
        case .earn: .none
        }
        let text: String? = switch providerType {
        case .stake: model.completionDateText
        case .earn: .none
        }
        guard let title, let text else { return nil }
        return ListItemField(title: title, value: text)
    }

    public var assetImageStyle: ListItemImageStyle? {
        .asset(assetImage: AssetViewModel(asset: asset).assetImage)
    }

    public var availableActions: [DelegationActionType] {
        service.delegationActions(walletType: wallet.type.map(), delegation: model.delegation.json())
            .map(DelegationActionType.init)
    }

    public var showManage: Bool {
        availableActions.isNotEmpty
    }

    public var canClaimRewards: Bool {
        service.canClaimDelegationRewards(walletType: wallet.type.map(), delegation: model.delegation.json())
    }

    public func actionTitle(_ action: DelegationActionType) -> String {
        switch action {
        case .stake: Localized.Transfer.Stake.title
        case .unstake: Localized.Transfer.Unstake.title
        case .redelegate: Localized.Transfer.Redelegate.title
        case .deposit: Localized.Wallet.deposit
        case .withdraw: Localized.Transfer.Withdraw.title
        case .claimRewards: Localized.Transfer.ClaimRewards.title
        }
    }
}

// MARK: - Actions

public extension DelegationSceneViewModel {
    func onSelectAction(_ action: DelegationActionType) {
        switch action {
        case .stake:
            onAmountInputAction?(amountInput(.stake(.stake(validators: validators, recommended: model.delegation.validator))))
        case .unstake:
            if stakeChain.canChangeAmountOnUnstake {
                onAmountInputAction?(amountInput(.stake(.unstake(model.delegation))))
            } else {
                onTransferAction?(stakeTransferData(.unstake(model.delegation)))
            }
        case .redelegate:
            onAmountInputAction?(amountInput(.stake(.redelegate(model.delegation, validators: validators, recommended: recommendedValidator))))
        case .deposit:
            onAmountInputAction?(amountInput(.earn(.deposit(model.delegation.validator))))
        case .withdraw:
            switch providerType {
            case .stake: onTransferAction?(stakeTransferData(.withdraw(model.delegation)))
            case .earn: onAmountInputAction?(amountInput(.earn(.withdraw(model.delegation))))
            }
        case .claimRewards:
            onClaimRewards()
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
        service.stakeTransferData(asset: asset.map(), stakeType: stakeType.json(), value: model.delegation.base.balanceValue.description, useMaxAmount: false)
    }

    private func claimRewardsTransferData() -> GemTransferData {
        service.stakeTransferData(
            asset: asset.map(),
            stakeType: StakeType.rewards([model.delegation.validator]).json(),
            value: model.delegation.base.rewardsValue.description,
            useMaxAmount: false,
        )
    }

    private var providerType: StakeProviderType {
        model.delegation.validator.providerType
    }

    private var stakeChain: StakeChain {
        StakeChain(rawValue: asset.chain.rawValue)!
    }

    private var recommendedValidator: DelegationValidator? {
        (try? service.recommendedValidator(
            chain: model.delegation.base.assetId.chain.rawValue,
            validators: validators.map { $0.json() },
        ).map { try DelegationValidator($0) }) ?? .none
    }
}
