// Copyright (c). Gem Wallet. All rights reserved.

import Components
import enum Gemstone.GemInfoImage
import GemstonePrimitives
import PrimitivesComponents
import Style
import SwiftUI

extension GemInfoImage {
    var sheetImage: InfoSheetImage {
        switch self {
        case .logo: .image(Images.Logo.logo)
        case .networkFee: .image(Images.Info.networkFee)
        case .watchWallet: .image(Images.Wallets.watch)
        case let .assetStatus(status): .assetImage(status.toPrimitives().statusAssetImage)
        case let .swapProvider(provider): .assetImage(AssetImage(placeholder: provider.toPrimitives().image))
        case let .asset(icon): .assetImage(AssetImage(icon: icon))
        case let .transactionState(icon, tone): .assetImage(AssetImage(icon: icon).badged(tone.image))
        }
    }
}

private extension AssetImage {
    func badged(_ badge: Image) -> AssetImage {
        AssetImage(imageURL: imageURL, placeholder: placeholder, chainPlaceholder: badge)
    }
}
