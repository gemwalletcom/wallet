// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import GemstonePrimitives
import Localization
import PrimitivesComponents

public struct SelectAssetPresentation: Sendable {
    public let title: String
    public let assetsSectionTitle: String

    init(
        title: String,
        assetsSectionTitle: String = Localized.Assets.title,
    ) {
        self.title = title
        self.assetsSectionTitle = assetsSectionTitle
    }
}

// MARK: - Models extensions

public extension SelectAssetType {
    func presentation() -> SelectAssetPresentation {
        switch self {
        case .send:
            SelectAssetPresentation(title: Localized.Wallet.send)
        case let .receive(type):
            switch type {
            case .asset:
                SelectAssetPresentation(title: Localized.Wallet.receive)
            case .collection:
                SelectAssetPresentation(
                    title: Localized.Wallet.receiveCollection,
                    assetsSectionTitle: Localized.Settings.Networks.title,
                )
            }
        case .buy:
            SelectAssetPresentation(title: Localized.Wallet.buy)
        case let .swap(type):
            switch type {
            case .pay:
                SelectAssetPresentation(title: Localized.Swap.youPay)
            case .receive:
                SelectAssetPresentation(title: Localized.Swap.youReceive)
            }
        case .manage:
            SelectAssetPresentation(title: Localized.Wallet.manageTokenList)
        case .priceAlert:
            SelectAssetPresentation(title: Localized.Assets.selectAsset)
        case .deposit:
            SelectAssetPresentation(title: Localized.Wallet.deposit)
        case .withdraw:
            SelectAssetPresentation(title: Localized.Wallet.withdraw)
        }
    }
}
