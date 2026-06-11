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
        [.asset, .perpetual]
    }

    static var recentActivityTypes: [RecentActivityType] {
        RecentActivityType.allCases
    }

    func searchMode(tag: String?) -> WalletSearchMode {
        if assetSearch.searchableQuery.isNotEmpty { return .searching }
        if tag != nil { return .tagBrowsing }
        return .initial
    }

    func assetsLimit(tag: String?) -> Int {
        switch searchMode(tag: tag) {
        case .initial: WalletSearchConfig.assetsInitialLimit
        case .tagBrowsing: WalletSearchConfig.assetsTagLimit
        case .searching: WalletSearchConfig.assetsSearchLimit
        }
    }

    func fetchLimit(tag: String?) -> Int {
        switch searchMode(tag: tag) {
        case .initial, .tagBrowsing: assetsLimit(tag: tag) + 1
        case .searching: WalletSearchConfig.resultsLimit
        }
    }
}
