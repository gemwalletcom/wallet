// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Foundation
import enum Gemstone.GemDelegationAction
import enum Gemstone.GemListRow
import protocol Gemstone.GemStakeServiceProtocol
import struct Gemstone.GemTransferData
import GemstonePrimitives
import Localization
import Primitives
import PrimitivesComponents
import Style
import SwiftUI

public struct DelegationSceneViewModel {
    public let model: DelegationViewModel
    public let validators: [DelegationValidator]
    public let onNavigate: StakeRouteAction

    private let wallet: Wallet
    private let asset: Asset
    private let service: any GemStakeServiceProtocol

    public init(
        wallet: Wallet,
        model: DelegationViewModel,
        asset: Asset,
        service: any GemStakeServiceProtocol,
        validators: [DelegationValidator],
        onNavigate: StakeRouteAction,
    ) {
        self.wallet = wallet
        self.model = model
        self.asset = asset
        self.service = service
        self.validators = validators
        self.onNavigate = onNavigate
    }

    public var title: String {
        providerType.title
    }

    public var rows: [GemListRow] {
        service.delegationRows(delegation: model.delegation.toGem())
    }

    public var rewardsItem: ListItemModel? {
        model.rewardsText.map { rewardsText in
            ListItemModel(
                title: Localized.Stake.rewards,
                titleStyle: model.titleStyle,
                subtitle: rewardsText,
                subtitleStyle: model.subtitleStyle,
                subtitleExtra: model.rewardsFiatValueText,
                subtitleStyleExtra: model.subtitleExtraStyle,
                imageStyle: assetImageStyle,
            )
        }
    }

    public func actionListItem(_ action: GemDelegationAction) -> ListItemModel {
        ListItemModel(title: action.title)
    }

    public var manageTitle: String {
        Localized.Common.manage
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
        let route = service.delegationActionDestination(asset: asset.toGem(), delegation: model.delegation.toGem(), action: action, validators: validators.map { $0.toGem() })
            .route(delegation: model.delegation, validators: validators)
        switch route {
        case .delegation: break
        case .transfer: onNavigate?(route)
        }
    }

    func onClaimRewards() {
        onNavigate?(.transfer(.confirm(claimRewardsTransferData())))
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
