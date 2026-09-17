// Copyright (c). Gem Wallet. All rights reserved.

import GemstonePrimitives
import struct Gemstone.GemRewardsRedemption
import struct Gemstone.RewardRedemptionOption
import Components
import Foundation
import Localization
import Primitives
import PrimitivesComponents

struct RewardRedemptionOptionViewModel: Identifiable {
    let redemption: GemRewardsRedemption

    private var option: RewardRedemptionOption {
        redemption.option
    }

    var id: String {
        option.id
    }

    var title: String {
        switch option.redemptionType {
        case .asset, .giftAsset:
            Localized.Rewards.WaysSpend.Asset.title(valueText)
        }
    }

    var subtitle: String {
        pointsText
    }

    var listItem: ListItemModel {
        ListItemModel(title: title, subtitle: subtitle, imageStyle: .asset(assetImage: assetImage))
    }

    var assetImage: AssetImage {
        guard let asset = option.asset else {
            return AssetImage()
        }
        return AssetIdViewModel(assetId: Primitives.AssetId(core: asset.id)).assetImage
    }

    var pointsText: String {
        redemption.pointsText
    }

    var canRedeem: Bool {
        redemption.canRedeem
    }

    var valueText: String {
        redemption.value.text()
    }

    var confirmationMessage: String {
        Localized.Rewards.confirmRedeem(valueText, pointsText)
    }
}
