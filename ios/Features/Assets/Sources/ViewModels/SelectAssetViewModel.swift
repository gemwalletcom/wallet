// Copyright (c). Gem Wallet. All rights reserved.

import GemstoneServices
import Components
import Foundation
import GemstonePrimitives
import Localization
import Preferences
import Primitives
import PrimitivesComponents
import Recents
import Store
import Style
import SwiftUI

@Observable
@MainActor
public final class SelectAssetViewModel {
    let preferences: Preferences
    let selectType: SelectAssetType
    let flow: SelectAssetFlow
    let searchService: AssetSearchService
    let assetsEnabler: any AssetsEnabler
    let priceAlertService: PriceAlertService
    let activityService: ActivityService

    public let wallet: Wallet

    var state: StateViewType<[AssetBasic]> = .noData
    var searchableQuery: String = .empty

    public let assetsQuery: ObservableQuery<AssetsRequest>
    public let recentModel: RecentAssetsModel
    var assets: [AssetData] {
        assetsQuery.value
    }

    var isPresentingCopyToast: Bool = false
    var copyTypeViewModel: CopyTypeViewModel?

    public var isPresentingAddToken: Bool = false
    public var assetSelection: AssetSelectionType?

    public var filterModel: AssetsFilterViewModel
    public var onSelectAssetAction: AssetAction

    public init(
        preferences: Preferences = Preferences.standard,
        wallet: Wallet,
        selectType: SelectAssetType,
        searchService: AssetSearchService,
        assetsEnabler: any AssetsEnabler,
        priceAlertService: PriceAlertService,
        activityService: ActivityService,
        selectAssetAction: AssetAction = .none,
        chains: [Chain] = [],
    ) {
        self.preferences = preferences
        self.wallet = wallet
        self.selectType = selectType
        self.searchService = searchService
        self.assetsEnabler = assetsEnabler
        self.priceAlertService = priceAlertService
        self.activityService = activityService
        flow = selectType.flow
        onSelectAssetAction = selectAssetAction

        let filter = AssetsFilterViewModel(
            type: selectType,
            model: ChainsFilterViewModel(
                chains: wallet.chains,
                selected: chains,
            ),
        )
        filterModel = filter

        assetsQuery = ObservableQuery(AssetsRequest(walletId: wallet.id, filters: filter.filters), initialValue: [])
        recentModel = RecentAssetsModel(
            walletId: wallet.id,
            types: selectType.recentActivityTypes,
            filters: filter.defaultFilters,
            activityService: activityService,
        )
    }

    var title: String {
        flow.title
    }

    var sections: AssetsSections {
        AssetsSections.from(assets, enablePopular: flow.capabilities.contains(.popularSection))
    }

    var showPopularSection: Bool {
        sections.popular.isNotEmpty
    }

    var showPinnedSection: Bool {
        sections.pinned.isNotEmpty
    }

    var showAssetsSection: Bool {
        sections.assets.isNotEmpty
    }

    var popularImage: Image {
        Images.System.starFill
    }

    var popularTitle: String {
        Localized.Assets.popular
    }

    var pinnedImage: Image {
        Images.System.pin
    }

    var pinnedTitle: String {
        Localized.Common.pinned
    }

    var assetsTitle: String {
        flow.assetsSectionTitle
    }

    public var showAddToken: Bool {
        flow.capabilities.contains(.addCustomToken) && wallet.hasTokenSupport && filterModel.chainsFilter.hasChains
    }

    public var showFilter: Bool {
        flow.capabilities.contains(.chainFilter) && wallet.isMultiCoins && filterModel.chainsFilter.hasChains
    }

    var isNetworkSearchEnabled: Bool {
        flow.capabilities.contains(.networkSearch)
    }

    var showLoading: Bool {
        state.isLoading && showEmpty
    }

    var showEmpty: Bool {
        sections.pinned.isEmpty && sections.assets.isEmpty
    }

    var showRecents: Bool {
        flow.capabilities.contains(.recents) && searchableQuery.isEmpty && recentModel.hasAssets
    }

    var currencyCode: String {
        preferences.currency
    }
}

// MARK: - Business Logic

extension SelectAssetViewModel {
    public func updateRecent(assetId: AssetId) {
        guard let data = selectType.recentActivityData(assetId: assetId) else { return }
        do {
            try activityService.updateRecent(data: data, walletId: wallet.id)
        } catch {
            debugLog("Failed to update recent activity: \(error)")
        }
    }

    func selectAsset(asset: Asset) {
        switch flow.selectionEffect {
        case .enablePriceAlert:
            Task {
                await setPriceAlert(assetId: asset.id, enabled: true)
            }
        case .recordRecent:
            updateRecent(assetId: asset.id)
        case .none:
            break
        }
        onSelectAssetAction?(asset)
    }

    func search(query: String) async {
        let query = query.trim()
        if query.isEmpty {
            return
        }
        await searchAssets(query: query)
    }

    func handleAction(assetId: AssetId, enabled: Bool) async {
        switch flow.rowSelection {
        case .toggle:
            do {
                try await assetsEnabler.enableAssets(wallet: wallet, assetIds: [assetId], enabled: enabled)
            } catch {
                debugLog("SelectAssetViewModel handleAction error: \(error)")
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
    }
}

// MARK: - Actions

extension SelectAssetViewModel {
    func onAssetAction(action: ListAssetItemAction, assetData: AssetData) {
        let asset = assetData.asset
        switch action {
        case let .switcher(enabled):
            Task {
                await handleAction(assetId: asset.id, enabled: enabled)
            }
        case .copy:
            let address = assetData.account.address
            copyTypeViewModel = CopyTypeViewModel(
                type: .address(asset, address: address),
                copyValue: address,
            )
            isPresentingCopyToast = true
            Task {
                await handleAction(assetId: asset.id, enabled: true)
            }
        }
    }

    func onSelectAsset(_ assetData: AssetData) {
        assetSelection = .regular(SelectAssetInput(type: selectType, assetData: assetData))
    }

    func displayAssetData(_ assetData: AssetData) -> AssetData {
        guard flow.capabilities.contains(.depositAssetDisplay) else { return assetData }
        return AssetData(
            asset: PerpetualConfig.depositAsset,
            balance: assetData.balance,
            account: assetData.account,
            price: assetData.price,
            priceAlerts: assetData.priceAlerts,
            metadata: assetData.metadata,
            associations: assetData.associations,
        )
    }

    public func onSelectRecent(_ asset: Asset) {
        switch flow.rowSelection {
        case .navigate:
            assetSelection = .recent(SelectAssetInput(type: selectType, assetData: assetData(for: asset)))
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
    private func assetData(for asset: Asset) -> AssetData {
        if let assetData = assets.first(where: { $0.asset.id == asset.id }) {
            return assetData
        }
        guard let account = try? wallet.account(for: asset.chain) else {
            return .with(asset: asset)
        }
        return .with(asset: asset, account: account)
    }

    private func searchAssets(query: String) async {
        do {
            let assets = try await searchService.searchAssets(wallet: wallet, query: query)
            state = .data(assets)
        } catch {
            handle(error: error)
        }
    }

    private func setPriceAlert(assetId: AssetId, enabled: Bool) async {
        do {
            let currency = try Currency(id: Preferences.standard.currency)
            if enabled {
                try await priceAlertService.enable(priceAlert: .default(for: assetId, currency: currency))
            } else {
                try await priceAlertService.delete(priceAlerts: [.default(for: assetId, currency: currency)])
            }
        } catch {
            handle(error: error)
        }
    }

    private func handle(error: any Error) {
        state.setError(error)
        debugLog("SelectAssetScene scene error: \(error)")
    }
}
