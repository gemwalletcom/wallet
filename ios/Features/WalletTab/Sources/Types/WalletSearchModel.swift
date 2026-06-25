// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import GemstonePrimitives
import Primitives
import PrimitivesComponents

enum WalletSearchMode {
    case initial
    case tagBrowsing
    case searching
}

struct WalletSearchModel {
    var assetSearch: AssetSearchViewModel

    var searchableQuery: String {
        get { assetSearch.searchableQuery }
        set { assetSearch.searchableQuery = newValue }
    }

    var tagsViewModel: AssetTagsViewModel {
        get { assetSearch.tagsViewModel }
        set { assetSearch.tagsViewModel = newValue }
    }

    var focus: AssetSearchViewModel.Focus {
        get { assetSearch.focus }
        set { assetSearch.focus = newValue }
    }

    init(selectType: SelectAssetType) {
        assetSearch = AssetSearchViewModel(selectType: selectType)
    }
}

// MARK: - Limits

extension WalletSearchModel {
    var perpetualsLimit: Int {
        WalletSearchConfig.perpetualsPreviewLimit
    }

    static var initialFetchLimit: Int {
        WalletSearchConfig.assetsInitialLimit + 1
    }

    static var searchItemTypes: [SearchItemType] {
        [.asset, .perpetual, .list]
    }

    static var recentActivityTypes: [RecentActivityType] {
        RecentActivityType.allCases
    }

    func searchMode(scope: WalletSearchTag) -> WalletSearchMode {
        if assetSearch.searchableQuery.isNotEmpty { return .searching }
        if !scope.isAll { return .tagBrowsing }
        return .initial
    }

    func assetsLimit(scope: WalletSearchTag) -> Int {
        switch searchMode(scope: scope) {
        case .initial: WalletSearchConfig.assetsInitialLimit
        case .tagBrowsing: WalletSearchConfig.assetsTagLimit
        case .searching: WalletSearchConfig.assetsSearchLimit
        }
    }

    func fetchLimit(scope: WalletSearchTag) -> Int {
        switch searchMode(scope: scope) {
        case .initial, .tagBrowsing: assetsLimit(scope: scope) + 1
        case .searching: WalletSearchConfig.resultsLimit
        }
    }
}
