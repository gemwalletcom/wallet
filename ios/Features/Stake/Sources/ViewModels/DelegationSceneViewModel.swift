// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Foundation
import enum Gemstone.GemDelegationAction
import struct Gemstone.GemDelegationDetails
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

    private var details: GemDelegationDetails {
        service.delegationDetails(
            delegation: model.delegation.toGem(),
            asset: asset.toGem(),
            price: model.delegation.price?.price,
            currency: model.currency.toGem(),
        )
    }

    public var title: String {
        details.title.text
    }

    public var rows: [GemListRow] {
        details.rows
    }

    public var rewardsItem: ListItemModel? {
        details.rewards.map { rewards in
            ListItemModel(
                title: Localized.Stake.rewards,
                titleStyle: model.titleStyle,
                subtitle: rewards.text(),
                subtitleStyle: model.subtitleStyle,
                subtitleExtra: details.rewardsFiat?.text(),
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
        guard let claim = details.claim else { return }
        onNavigate?(.transfer(.confirm(claim)))
    }
}

extension GemDelegationAction: @retroactive Identifiable {
    public var id: Self { self }
}
