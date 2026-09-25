// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Foundation
import func Gemstone.addressCopy
import struct Gemstone.GemAssetSectionCounts
import protocol Gemstone.GemAssetSelectionServiceProtocol
import protocol Gemstone.GemRecentActivityServiceProtocol
import struct Gemstone.GemSelectAssetFlow
import enum Gemstone.GemSelectAssetState
import struct Gemstone.GemSelectAssetWalletFlow
import enum Gemstone.GemServiceError
import GemstonePrimitives
import GemstoneServices
import Localization
import Primitives
import PrimitivesComponents
import Recents
import Store
import Style
import SwiftUI

@Observable
@MainActor
public final class SelectAssetViewModel {
    private let service: any GemAssetSelectionServiceProtocol
    let selectType: SelectAssetType
    let flow: GemSelectAssetFlow
    private let walletFlow: GemSelectAssetWalletFlow

    public let wallet: Wallet

    var state: StateViewType<[AssetBasic]> = .noData
    var searchableQuery: String = .empty

    public let assetsQuery: ObservableQuery<AssetsRequest>
    public let recentModel: RecentAssetsModel
    var assets: [AssetData] {
        assetsQuery.value
    }

    var copyToast: CopyTypeViewModel?
    var isPresentingToastMessage: ToastMessage?

    public var isPresentingAddToken: Bool = false
    public var assetSelection: SelectAssetInput?

    public var filterModel: AssetsFilterViewModel
    public var onSelectAssetAction: AssetAction

    public init(
        wallet: Wallet,
        selectType: SelectAssetType,
        service: any GemAssetSelectionServiceProtocol,
        recentAssetsService: any GemRecentActivityServiceProtocol,
        selectAssetAction: AssetAction = .none,
        chains: [Chain] = [],
    ) {
        self.service = service
        self.wallet = wallet
        self.selectType = selectType
        let walletFlow = service.walletFlow(selectType: selectType.flowType, wallet: wallet.toGem())
        self.walletFlow = walletFlow
        flow = walletFlow.flow
        onSelectAssetAction = selectAssetAction

        let filter = AssetsFilterViewModel(
            flow: flow,
            model: ChainsFilterViewModel(
                chains: walletFlow.chains.map { Chain(core: $0) },
                selected: chains,
            ),
        )
        filterModel = filter

        assetsQuery = ObservableQuery(AssetsRequest(walletId: wallet.id, scope: flow.requestScope, filters: filter.filters), initialValue: [])
        recentModel = RecentAssetsModel(
            walletId: wallet.id,
            types: flow.action?.recentActivityTypes().map { $0.toPrimitives() } ?? RecentActivityType.allCases,
            filters: filter.filters,
            service: recentAssetsService,
        )
    }

    var title: String {
        flow.title.text
    }

    var sections: AssetsSections {
        AssetsSections.from(assets, showsPopular: flow.popularSection)
    }

    var popularImage: Image {
        Images.System.starFill
    }

    var popularTitle: String {
        Localized.Assets.popular
    }

    var assetsTitle: String {
        flow.assetsSection.text
    }

    public var showAddToken: Bool {
        walletFlow.showsAddToken
    }

    public var showFilter: Bool {
        walletFlow.showsChainFilter
    }

    var isNetworkSearchEnabled: Bool {
        flow.networkSearch
    }

    func listState(_ sections: AssetsSections) -> GemSelectAssetState {
        let counts = GemAssetSectionCounts(
            pinned: UInt32(sections.pinned.count),
            popular: UInt32(sections.popular.count),
            assets: UInt32(sections.assets.count),
        )
        return flow.state(counts: counts, isSearching: state.isLoading)
    }

    var showRecents: Bool {
        flow.showsRecents(isSearching: !searchableQuery.isEmpty, hasRecents: recentModel.hasAssets)
    }

    var assetItems: ListAssetItemsViewModel {
        ListAssetItemsViewModel(currency: service.getCurrency().toPrimitives(), rowStyle: flow.rowStyle)
    }
}

// MARK: - Business Logic

extension SelectAssetViewModel {
    func selectAsset(asset: Asset) {
        recordSelection(asset: asset)
        onSelectAssetAction?(asset)
    }

    func search(query: String) async {
        switch flow.searchStep(query: query) {
        case .idle: break
        case let .search(query): await searchAssets(query: query)
        }
    }

    func setAssetEnabled(assetId: AssetId, enabled: Bool) async {
        switch flow.rowAction {
        case .toggle:
            do {
                try await service.setAssetsEnabled(assetIds: [assetId.identifier], enabled: enabled)
            } catch let error as GemServiceError {
                isPresentingToastMessage = .error(error.text().text)
            } catch {
                debugLog("SelectAssetViewModel set asset enabled error: \(error)")
            }
        case .navigate, .select:
            break
        }
    }

    func updateRequest() {
        assetsQuery.request.searchBy = searchableQuery
        state = isNetworkSearchEnabled ? .loading : .noData
    }

    func onChangeFilterModel(_: AssetsFilterViewModel, model: AssetsFilterViewModel) {
        assetsQuery.request.filters = model.filters
        recentModel.query.request.filters = model.filters
    }
}

// MARK: - Actions

extension SelectAssetViewModel {
    func onAssetAction(action: ListAssetItemAction, assetData: AssetData) {
        let asset = assetData.asset
        switch action {
        case let .switcher(enabled):
            Task {
                await setAssetEnabled(assetId: asset.id, enabled: enabled)
            }
        case .copy:
            let address = assetData.account.address
            copyToast = CopyTypeViewModel(content: addressCopy(chain: asset.chain.toGem(), address: address))
            Task {
                await setAssetEnabled(assetId: asset.id, enabled: true)
            }
        }
    }

    func onSelectAsset(_ assetData: AssetData) {
        recordSelection(asset: assetData.asset)
        assetSelection = SelectAssetInput(type: selectType, assetData: assetData)
    }

    func displayAssetData(_ assetData: AssetData) -> AssetData {
        guard let displayAsset = flow.displayAsset else { return assetData }
        return AssetData(
            asset: displayAsset.toPrimitives(),
            balance: assetData.balance,
            account: assetData.account,
            price: assetData.price,
            priceAlerts: assetData.priceAlerts,
            metadata: assetData.metadata,
            associations: assetData.associations,
        )
    }

    public func onSelectRecent(_ asset: Asset) {
        switch flow.rowAction {
        case .navigate:
            assetSelection = assetData(for: asset).map { SelectAssetInput(type: selectType, assetData: $0) }
        case .select:
            onSelectAssetAction?(asset)
        case .toggle:
            break
        }
        recentModel.dismiss()
    }

    func onSelectAddCustomToken() {
        isPresentingAddToken.toggle()
    }
}

// MARK: - Private

extension SelectAssetViewModel {
    private func recordSelection(asset: Asset) {
        if flow.enablesPriceAlert {
            Task {
                await setPriceAlert(assetId: asset.id, enabled: true)
            }
        }
        if let action = flow.action {
            Task { [service] in
                do {
                    try await service.addRecent(action: action, asset: asset.toGem())
                } catch {
                    debugLog("Failed to update recent activity: \(error)")
                }
            }
        }
    }

    private func assetData(for asset: Asset) -> AssetData? {
        if let assetData = assets.first(where: { $0.asset.id == asset.id }) {
            return assetData
        }
        guard let account = try? wallet.account(for: asset.chain) else {
            return nil
        }
        return .with(asset: asset, account: account)
    }

    private func searchAssets(query: String) async {
        do {
            let assets = try await service.searchAssets(query: query).map { $0.toPrimitives() }
            guard flow.searchStep(query: searchableQuery) == .search(query: query) else { return }
            state = .data(assets)
        } catch {
            guard flow.searchStep(query: searchableQuery) == .search(query: query) else { return }
            showError(error)
        }
    }

    private func setPriceAlert(assetId: AssetId, enabled: Bool) async {
        do {
            try await service.setPriceAlert(assetId: assetId.identifier, enabled: enabled)
        } catch {
            showError(error)
        }
    }

    private func showError(_ error: any Error) {
        state.setError(error)
        debugLog("SelectAssetScene scene error: \(error)")
    }
}
