// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Foundation
import struct Gemstone.GemDelegationActionItem
import struct Gemstone.GemDelegationDetails
import enum Gemstone.GemListRow
import protocol Gemstone.GemStakeServiceProtocol
import struct Gemstone.GemTransferData
import GemstonePrimitives
import Localization
import Primitives
import PrimitivesComponents
import Store
import Style
import SwiftUI

@MainActor
@Observable
public final class DelegationSceneViewModel {
    public let delegation: Delegation
    public let onNavigate: StakeRouteAction
    public let validatorsQuery: ObservableQuery<ValidatorsQuery>

    private let wallet: Wallet
    private let asset: Asset
    private let service: any GemStakeServiceProtocol
    private let onSelectAddress: (@MainActor @Sendable (ChainAddress) -> Void)?

    public init(
        wallet: Wallet,
        delegation: Delegation,
        asset: Asset,
        service: any GemStakeServiceProtocol,
        onNavigate: StakeRouteAction,
        onSelectAddress: (@MainActor @Sendable (ChainAddress) -> Void)? = nil,
    ) {
        self.wallet = wallet
        self.delegation = delegation
        self.asset = asset
        self.service = service
        self.onNavigate = onNavigate
        self.onSelectAddress = onSelectAddress
        validatorsQuery = ObservableQuery(ValidatorsQuery(chain: delegation.validator.chain, providerType: .stake), initialValue: [])
    }

    @MainActor
    var onSelectProvider: ((String) -> Void)? {
        guard let onSelectAddress else { return nil }
        let chain = delegation.validator.chain
        return { onSelectAddress(ChainAddress(chain: chain, address: $0)) }
    }

    public var details: GemDelegationDetails {
        service.delegationDetails(
            walletType: wallet.type.toGem(),
            delegation: delegation.toGem(),
            asset: asset.toGem(),
            price: price,
            currency: service.getCurrency(),
            validators: validatorsQuery.value.map { $0.toGem() },
        )
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

    public func actionListItem(_ item: GemDelegationActionItem) -> ListItemModel {
        ListItemModel(title: item.action.title)
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
    func onSelectAction(_ item: GemDelegationActionItem) {
        let route = item.destination.route(delegation: delegation)
        switch route {
        case .delegation: break
        case .transfer: onNavigate?(route)
        }
    }

    func onClaimRewards(_ claim: GemTransferData) {
        onNavigate?(.transfer(.confirm(claim)))
    }
}
