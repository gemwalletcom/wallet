// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Foundation
import struct Gemstone.GemRewardsRedemption
import GemstonePrimitives
import Localization
import Primitives
import PrimitivesComponents

struct RewardRedemptionOptionViewModel: Identifiable {
    let redemption: GemRewardsRedemption

    var id: String {
        redemption.id
    }

    var title: String {
        redemption.title.text
    }

    var subtitle: String {
        pointsText
    }

    var listItem: ListItemModel {
        ListItemModel(title: title, subtitle: subtitle, imageStyle: .asset(assetImage: assetImage))
    }

    var assetImage: AssetImage {
        AssetIdViewModel(assetId: Primitives.AssetId(core: redemption.assetId)).assetImage
    }

    var pointsText: String {
        redemption.points.text()
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
