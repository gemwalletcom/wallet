// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import GemstonePrimitives
import Localization
import PrimitivesComponents

public struct SelectAssetPresentation: Sendable {
    public let title: String
    public let assetsSectionTitle: String
    public let listType: AssetListType

    init(
        title: String,
        assetsSectionTitle: String = Localized.Assets.title,
        listType: AssetListType,
    ) {
        self.title = title
        self.assetsSectionTitle = assetsSectionTitle
        self.listType = listType
    }
}

// MARK: - Models extensions

public extension SelectAssetType {
    func presentation() -> SelectAssetPresentation {
        switch self {
        case .send:
            SelectAssetPresentation(title: Localized.Wallet.send, listType: .view)
        case let .receive(type):
            switch type {
            case .asset:
                SelectAssetPresentation(title: Localized.Wallet.receive, listType: .copy(.asset))
            case .collection:
                SelectAssetPresentation(
                    title: Localized.Wallet.receiveCollection,
                    assetsSectionTitle: Localized.Settings.Networks.title,
                    listType: .copy(.collection),
                )
            }
        case .buy:
            SelectAssetPresentation(title: Localized.Wallet.buy, listType: .view)
        case let .swap(type):
            switch type {
            case .pay:
                SelectAssetPresentation(title: Localized.Swap.youPay, listType: .view)
            case .receive:
                SelectAssetPresentation(title: Localized.Swap.youReceive, listType: .view)
            }
        case .manage:
            SelectAssetPresentation(title: Localized.Wallet.manageTokenList, listType: .manage)
        case .priceAlert:
            SelectAssetPresentation(title: Localized.Assets.selectAsset, listType: .price)
        case .deposit:
            SelectAssetPresentation(title: Localized.Wallet.deposit, listType: .view)
        case .withdraw:
            SelectAssetPresentation(title: Localized.Wallet.withdraw, listType: .view)
        }
    }
}
