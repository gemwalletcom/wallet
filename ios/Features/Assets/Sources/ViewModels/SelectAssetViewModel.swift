// Copyright (c). Gem Wallet. All rights reserved.

import protocol Gemstone.GemChainServiceProtocol
import Components
import protocol Gemstone.GemAssetSelectionServiceProtocol
import protocol Gemstone.GemRecentActivityServiceProtocol
import GemstoneServices
import Foundation
import class Gemstone.GemAssetConfigService
import GemstonePrimitives
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
    private let chainService: any GemChainServiceProtocol
    private let recentAssetsService: any GemRecentActivityServiceProtocol
    let selectType: SelectAssetType
    let flow: SelectAssetFlow

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
    public var assetSelection: SelectAssetInput?

    public var filterModel: AssetsFilterViewModel
    public var onSelectAssetAction: AssetAction

    public init(
        wallet: Wallet,
        selectType: SelectAssetType,
        service: any GemAssetSelectionServiceProtocol,
        chainService: any GemChainServiceProtocol,
        recentAssetsService: any GemRecentActivityServiceProtocol,
        selectAssetAction: AssetAction = .none,
        chains: [Chain] = [],
    ) {
        self.service = service
        self.wallet = wallet
        self.selectType = selectType
        self.chainService = chainService
        self.recentAssetsService = recentAssetsService
        flow = selectType.flow()
        onSelectAssetAction = selectAssetAction

        let filter = AssetsFilterViewModel(
            type: selectType,
            model: ChainsFilterViewModel(
                chains: wallet.chains,
                selected: chains,
            ),
            chainService: chainService,
        )
        filterModel = filter

        assetsQuery = ObservableQuery(AssetsRequest(walletId: wallet.id, filters: filter.filters), initialValue: [])
        recentModel = RecentAssetsModel(
            walletId: wallet.id,
            types: selectType.recentActivityTypes,
            filters: filter.defaultFilters,
            recentAssetsService: recentAssetsService,
        )
    }

    var title: String {
        flow.title
    }

    var sections: AssetsSections {
        AssetsSections.from(assets, popularIds: flow.capabilities.contains(.popularSection) ? Self.popularIds : [])
    }

    private static let popularIds = Set(GemAssetConfigService().popularIds().compactMap { try? AssetId(id: $0) })

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
        service.currency()
    }
}

// MARK: - Business Logic

extension SelectAssetViewModel {
    func selectAsset(asset: Asset) {
        applySelectionEffect(assetId: asset.id)
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
                try await service.setAssetsEnabled(walletId: wallet.id.id, assetIds: [assetId.identifier], enabled: enabled)
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
        applySelectionEffect(assetId: assetData.asset.id)
        assetSelection = SelectAssetInput(type: selectType, assetData: assetData)
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
            assetSelection = SelectAssetInput(type: selectType, assetData: assetData(for: asset))
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
    private func applySelectionEffect(assetId: AssetId) {
        switch flow.selectionEffect {
        case .enablePriceAlert:
            Task {
                await setPriceAlert(assetId: assetId, enabled: true)
            }
        case .recordRecent:
            updateRecent(assetId: assetId)
        case .none:
            break
        }
    }

    private func updateRecent(assetId: AssetId) {
        guard let data = selectType.recentActivityData(assetId: assetId) else { return }
        Task { [service, wallet] in
            do {
                try await service.addRecentAsset(activityType: data.type.map(), assetId: data.assetId.identifier, walletId: wallet.id.id)
            } catch {
                debugLog("Failed to update recent activity: \(error)")
            }
        }
    }

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
            let assets = try await service.searchAssets(wallet: wallet.json(), query: query).map { try AssetBasic($0) }
            state = .data(assets)
        } catch {
            handle(error: error)
        }
    }

    private func setPriceAlert(assetId: AssetId, enabled: Bool) async {
        do {
            try await service.setPriceAlert(assetId: assetId.identifier, enabled: enabled)
        } catch {
            handle(error: error)
        }
    }

    private func handle(error: any Error) {
        state.setError(error)
        debugLog("SelectAssetScene scene error: \(error)")
    }
}
