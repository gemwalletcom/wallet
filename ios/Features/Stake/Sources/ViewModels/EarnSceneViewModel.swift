// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Foundation
import struct Gemstone.GemEarnInput
import struct Gemstone.GemEarnView
import enum Gemstone.GemLoadState
import struct Gemstone.GemStakeDelegationItem
import protocol Gemstone.GemStakeServiceProtocol
import GemstonePrimitives
import GemstoneServices
import Localization
import Primitives
import PrimitivesComponents
import Store

@MainActor
@Observable
public final class EarnSceneViewModel {
    private let service: any GemStakeServiceProtocol
    private let onNavigate: StakeRouteAction
    private var viewState: GemLoadState = .loading

    public let wallet: Wallet
    public let asset: Asset

    public let assetQuery: ObservableQuery<AssetQuery>
    public let positionsQuery: ObservableQuery<DelegationsQuery>
    public let providersQuery: ObservableQuery<ValidatorsQuery>

    public var assetData: AssetData {
        assetQuery.value
    }

    public init(
        wallet: Wallet,
        asset: Asset,
        service: any GemStakeServiceProtocol,
        onNavigate: StakeRouteAction,
    ) {
        self.wallet = wallet
        self.asset = asset
        self.service = service
        self.onNavigate = onNavigate
        assetQuery = ObservableQuery(AssetQuery(walletId: wallet.id, assetId: asset.id), initialValue: .with(asset: asset))
        positionsQuery = ObservableQuery(
            DelegationsQuery(walletId: wallet.id, assetId: asset.id, providerType: .earn),
            initialValue: [],
        )
        providersQuery = ObservableQuery(
            ValidatorsQuery(chain: asset.id.chain, providerType: .earn),
            initialValue: [],
        )
    }

    var title: String {
        Localized.Common.earn
    }

    var earnView: GemEarnView {
        service.earnView(input: GemEarnInput(
            walletType: wallet.type.toGem(),
            asset: asset.toGem(),
            providers: providersQuery.value.map { $0.toGem() },
            delegations: positionsQuery.value.map { $0.toGem() },
            assetApr: assetData.metadata.earnApr,
            price: assetData.price?.price,
            currency: service.getCurrency(),
            state: viewState,
        ))
    }

    func depositRoute(_ view: GemEarnView) -> StakeRoute? {
        view.depositProvider.map { .transfer(.amount(AmountInput(type: .earn(.deposit($0)), asset: asset))) }
    }

    var emptyContentModel: EmptyStateViewModel {
        EmptyStateViewModel(kind: .earn, symbol: asset.symbol)
    }
}

// MARK: - Actions

extension EarnSceneViewModel {
    func onSelect(item: GemStakeDelegationItem) {
        onNavigate?(item.destination.route(delegation: item.delegation.toPrimitives()))
    }

    func onSelectDeposit() {
        depositRoute(earnView).map { onNavigate?($0) }
    }

    func load() async {
        viewState = .loading
        viewState = await service.refreshEarn(assetId: asset.id.identifier, hasRows: earnView.positions.isNotEmpty)
    }
}
