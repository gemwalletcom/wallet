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
    public let delegation: Delegation
    public let validators: [DelegationValidator]
    public let onNavigate: StakeRouteAction

    private let wallet: Wallet
    private let asset: Asset
    private let service: any GemStakeServiceProtocol
    private let onSelectAddress: (@MainActor @Sendable (ChainAddress) -> Void)?

    public init(
        wallet: Wallet,
        delegation: Delegation,
        asset: Asset,
        service: any GemStakeServiceProtocol,
        validators: [DelegationValidator],
        onNavigate: StakeRouteAction,
        onSelectAddress: (@MainActor @Sendable (ChainAddress) -> Void)? = nil,
    ) {
        self.wallet = wallet
        self.delegation = delegation
        self.asset = asset
        self.service = service
        self.validators = validators
        self.onNavigate = onNavigate
        self.onSelectAddress = onSelectAddress
    }

    @MainActor
    var onSelectProvider: ((String) -> Void)? {
        guard let onSelectAddress else { return nil }
        let chain = delegation.validator.chain
        return { onSelectAddress(ChainAddress(chain: chain, address: $0)) }
    }

    public var details: GemDelegationDetails {
        service.delegationDetails(walletType: wallet.type.toGem(), delegation: delegation.toGem(), asset: asset.toGem(), price: price, currency: service.getCurrency())
    }

    public func header(_ details: GemDelegationDetails) -> ValueHeader {
        DelegationViewModel(row: details.header).header
    }

    private var price: Double? {
        delegation.price?.price
    }

    public func rewardsItem(_ details: GemDelegationDetails) -> ListItemModel? {
        let model = DelegationViewModel(row: details.header)
        return details.rewards.map { rewards in
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
        .asset(assetImage: AssetIdViewModel(assetId: asset.id).assetImage)
    }
}

// MARK: - Actions

public extension DelegationSceneViewModel {
    func onSelectAction(_ action: GemDelegationAction) {
        let route = service.delegationActionDestination(asset: asset.toGem(), delegation: delegation.toGem(), action: action, validators: validators.map { $0.toGem() })
            .route(delegation: delegation, validators: validators)
        switch route {
        case .delegation: break
        case .transfer: onNavigate?(route)
        }
    }

    func onClaimRewards(_ claim: GemTransferData) {
        onNavigate?(.transfer(.confirm(claim)))
    }
}

extension GemDelegationAction: @retroactive Identifiable {
    public var id: Self { self }
}
