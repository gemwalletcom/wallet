// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import GemstonePrimitives
import Localization
import Primitives
import PrimitivesComponents
import Store

public struct SelectAssetFlow: Sendable {
    public enum RowSelection: Sendable, Equatable {
        case navigate
        case toggle
        case select
    }

    public enum SelectionEffect: Sendable, Equatable {
        case enablePriceAlert
        case recordRecent
        case none
    }

    public struct Capabilities: OptionSet, Sendable {
        public let rawValue: Int

        public init(rawValue: Int) {
            self.rawValue = rawValue
        }

        public static let networkSearch = Capabilities(rawValue: 1 << 0)
        public static let chainFilter = Capabilities(rawValue: 1 << 1)
        public static let recents = Capabilities(rawValue: 1 << 2)
        public static let popularSection = Capabilities(rawValue: 1 << 3)
        public static let balanceFilter = Capabilities(rawValue: 1 << 4)
        public static let addCustomToken = Capabilities(rawValue: 1 << 5)
        public static let depositAssetDisplay = Capabilities(rawValue: 1 << 6)
    }

    public let title: String
    public let assetsSectionTitle: String
    public let listType: AssetListType
    public let defaultFilters: [AssetsRequestFilter]
    public let rowSelection: RowSelection
    public let selectionEffect: SelectionEffect
    public let capabilities: Capabilities

    init(
        title: String,
        assetsSectionTitle: String = Localized.Assets.title,
        listType: AssetListType,
        defaultFilters: [AssetsRequestFilter],
        rowSelection: RowSelection,
        selectionEffect: SelectionEffect = .none,
        capabilities: Capabilities = [],
    ) {
        self.title = title
        self.assetsSectionTitle = assetsSectionTitle
        self.listType = listType
        self.defaultFilters = defaultFilters
        self.rowSelection = rowSelection
        self.selectionEffect = selectionEffect
        self.capabilities = capabilities
    }
}

// MARK: - Models extensions

public extension SelectAssetType {
    var flow: SelectAssetFlow {
        switch self {
        case .send:
            SelectAssetFlow(
                title: Localized.Wallet.send,
                listType: .view,
                defaultFilters: [.enabled, .hasBalance],
                rowSelection: .navigate,
                capabilities: [.chainFilter, .recents],
            )
        case let .receive(type):
            switch type {
            case .asset:
                SelectAssetFlow(
                    title: Localized.Wallet.receive,
                    listType: .copy(.asset),
                    defaultFilters: [.enabled],
                    rowSelection: .navigate,
                    selectionEffect: .recordRecent,
                    capabilities: [.networkSearch, .chainFilter, .recents],
                )
            case .collection:
                SelectAssetFlow(
                    title: Localized.Wallet.receiveCollection,
                    assetsSectionTitle: Localized.Settings.Networks.title,
                    listType: .copy(.collection),
                    defaultFilters: [
                        .enabled,
                        .chainsOrAssets([], Chain.allCases.filter(\.isNFTSupported).map(\.rawValue)),
                    ],
                    rowSelection: .navigate,
                    selectionEffect: .recordRecent,
                    capabilities: [.networkSearch, .recents],
                )
            }
        case .buy:
            SelectAssetFlow(
                title: Localized.Wallet.buy,
                listType: .view,
                defaultFilters: [.enabled, .buyable],
                rowSelection: .navigate,
                selectionEffect: .recordRecent,
                capabilities: [.networkSearch, .chainFilter, .recents, .popularSection],
            )
        case let .swap(type):
            switch type {
            case .pay:
                SelectAssetFlow(
                    title: Localized.Swap.youPay,
                    listType: .view,
                    defaultFilters: [.enabled, .swappable, .hasBalance],
                    rowSelection: .select,
                    selectionEffect: .recordRecent,
                    capabilities: [.chainFilter, .recents],
                )
            case let .receive(chains, assetIds):
                SelectAssetFlow(
                    title: Localized.Swap.youReceive,
                    listType: .view,
                    defaultFilters: [
                        .enabled,
                        .chainsOrAssets(chains.map(\.rawValue), assetIds.map(\.identifier)),
                        .swappable,
                    ],
                    rowSelection: .select,
                    selectionEffect: .recordRecent,
                    capabilities: [.networkSearch, .chainFilter, .recents],
                )
            }
        case .manage:
            SelectAssetFlow(
                title: Localized.Wallet.manageTokenList,
                listType: .manage,
                defaultFilters: [.enabled],
                rowSelection: .toggle,
                capabilities: [.networkSearch, .chainFilter, .balanceFilter, .addCustomToken],
            )
        case .priceAlert:
            SelectAssetFlow(
                title: Localized.Assets.selectAsset,
                listType: .price,
                defaultFilters: [.enabled, .priceAlerts],
                rowSelection: .select,
                selectionEffect: .enablePriceAlert,
                capabilities: [.networkSearch, .chainFilter, .popularSection],
            )
        case .deposit:
            SelectAssetFlow(
                title: Localized.Wallet.deposit,
                listType: .view,
                defaultFilters: [.chainsOrAssets([], [PerpetualConfig.depositAssetId])],
                rowSelection: .navigate,
            )
        case .withdraw:
            SelectAssetFlow(
                title: Localized.Wallet.withdraw,
                listType: .view,
                defaultFilters: [.chainsOrAssets([], [Chain.hyperCore.defaultAsset(type: .perpetual).id.identifier])],
                rowSelection: .navigate,
                capabilities: [.depositAssetDisplay],
            )
        }
    }
}
