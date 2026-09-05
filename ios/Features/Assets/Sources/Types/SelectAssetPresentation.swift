// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import class Gemstone.Config
import enum Gemstone.GemAssetAction
import GemstonePrimitives
import Localization
import Primitives
import PrimitivesComponents
import Store

public struct SelectAssetPresentation: Sendable {
    public let title: String
    public let assetsSectionTitle: String
    public let listType: AssetListType
    public let defaultFilters: [AssetsRequestFilter]

    init(
        title: String,
        assetsSectionTitle: String = Localized.Assets.title,
        listType: AssetListType,
        defaultFilters: [AssetsRequestFilter],
    ) {
        self.title = title
        self.assetsSectionTitle = assetsSectionTitle
        self.listType = listType
        self.defaultFilters = defaultFilters
    }
}

private extension [AssetsRequestFilter] {
    static func filters(for action: GemAssetAction) -> [AssetsRequestFilter] {
        action.filters().map { filter in
            switch filter {
            case .enabled: .enabled
            case .buyable: .buyable
            case .sellable: .sellable
            case .swappable: .swappable
            case .hasBalance: .hasBalance
            case .hasAvailableBalance: .hasAvailableBalance
            }
        }
    }
}

// MARK: - Models extensions

public extension SelectAssetType {
    func presentation() -> SelectAssetPresentation {
        switch self {
        case .send:
            SelectAssetPresentation(
                title: Localized.Wallet.send,
                listType: .view,
                defaultFilters: .filters(for: .send),
            )
        case let .receive(type):
            switch type {
            case .asset:
                SelectAssetPresentation(
                    title: Localized.Wallet.receive,
                    listType: .copy(.asset),
                    defaultFilters: [.enabled],
                )
            case .collection:
                SelectAssetPresentation(
                    title: Localized.Wallet.receiveCollection,
                    assetsSectionTitle: Localized.Settings.Networks.title,
                    listType: .copy(.collection),
                    defaultFilters: [
                        .enabled,
                        .chainsOrAssets([], Config().getNftChains()),
                    ],
                )
            }
        case .buy:
            SelectAssetPresentation(
                title: Localized.Wallet.buy,
                listType: .view,
                defaultFilters: .filters(for: .buy),
            )
        case let .swap(type):
            switch type {
            case .pay:
                SelectAssetPresentation(
                    title: Localized.Swap.youPay,
                    listType: .view,
                    defaultFilters: .filters(for: .swapPay),
                )
            case let .receive(chains, assetIds):
                SelectAssetPresentation(
                    title: Localized.Swap.youReceive,
                    listType: .view,
                    defaultFilters: .filters(for: .swapReceive) + [.chainsOrAssets(chains.map(\.rawValue), assetIds.map(\.identifier))],
                )
            }
        case .manage:
            SelectAssetPresentation(
                title: Localized.Wallet.manageTokenList,
                listType: .manage,
                defaultFilters: [.enabled],
            )
        case .priceAlert:
            SelectAssetPresentation(
                title: Localized.Assets.selectAsset,
                listType: .price,
                defaultFilters: [.enabled, .priceAlerts],
            )
        case .deposit:
            SelectAssetPresentation(
                title: Localized.Wallet.deposit,
                listType: .view,
                defaultFilters: [.chainsOrAssets([], [PerpetualConfig.depositAssetId])],
            )
        case .withdraw:
            SelectAssetPresentation(
                title: Localized.Wallet.withdraw,
                listType: .view,
                defaultFilters: [.chainsOrAssets([], [Chain.hyperCore.defaultAsset(type: .perpetual).id.identifier])],
            )
        }
    }
}
