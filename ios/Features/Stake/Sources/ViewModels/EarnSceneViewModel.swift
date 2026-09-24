// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Foundation
import struct Gemstone.GemEarnView
import enum Gemstone.GemLoadState
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

    public let assetQuery: ObservableQuery<AssetRequest>
    public let positionsQuery: ObservableQuery<DelegationsRequest>
    public let providersQuery: ObservableQuery<ValidatorsRequest>

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
        assetQuery = ObservableQuery(AssetRequest(walletId: wallet.id, assetId: asset.id), initialValue: .with(asset: asset))
        positionsQuery = ObservableQuery(
            DelegationsRequest(walletId: wallet.id, assetId: asset.id, providerType: .earn),
            initialValue: [],
        )
        providersQuery = ObservableQuery(
            ValidatorsRequest(chain: asset.id.chain, providerType: .earn),
            initialValue: [],
        )
    }

    var title: String {
        Localized.Common.earn
    }

    var assetModel: AssetViewModel {
        AssetViewModel(asset: asset)
    }

    var earnView: GemEarnView {
        service.earnView(
            walletType: wallet.type.toGem(),
            providers: providersQuery.value.map { $0.toGem() },
            delegations: positionsQuery.value.map { $0.toGem() },
            assetApr: assetData.metadata.earnApr,
        )
    }

    var noDataListItem: ListItemModel {
        ListItemModel(title: Localized.Errors.noDataAvailable)
    }

    var depositListItem: ListItemModel {
        ListItemModel(title: Localized.Wallet.deposit)
    }

    func depositRoute(_ view: GemEarnView) -> StakeRoute? {
        view.depositProvider.map { .transfer(.amount(AmountInput(type: .earn(.deposit($0)), asset: asset))) }
    }

    var emptyContentModel: EmptyContentTypeViewModel {
        EmptyContentTypeViewModel(type: EmptyContentType(.earn, symbol: asset.symbol))
    }

    func positionItems(_ view: GemEarnView) -> [(delegation: Delegation, model: DelegationViewModel)] {
        DelegationViewModel.items(view.positions.map { $0.toPrimitives() }, asset: asset, price: assetData.price?.price, currency: service.getCurrency().toPrimitives())
    }

    func route(delegation: Delegation) -> StakeRoute {
        service.delegationDestination(walletType: wallet.type.toGem(), asset: asset.toGem(), delegation: delegation.toGem())
            .route(delegation: delegation, validators: [])
    }

    func showsEmptyState(_ view: GemEarnView) -> Bool {
        view.positions.isEmpty && viewState != .loading
    }

    func positionsSectionTitle(_ view: GemEarnView) -> String {
        view.positions.isEmpty ? .empty : Localized.Perpetual.positions
    }

    func providersState(_ view: GemEarnView) -> StateViewType<Bool> {
        viewState.stateViewType(view.providers).map { _ in true }
    }
}

// MARK: - Actions

extension EarnSceneViewModel {
    func onSelect(delegation: Delegation) {
        onNavigate?(route(delegation: delegation))
    }

    func onSelectDeposit() {
        depositRoute(earnView).map { onNavigate?($0) }
    }

    func load() async {
        viewState = .loading
        viewState = await service.refreshEarn(assetId: asset.id.identifier, hasRows: earnView.positions.isNotEmpty)
    }
}
