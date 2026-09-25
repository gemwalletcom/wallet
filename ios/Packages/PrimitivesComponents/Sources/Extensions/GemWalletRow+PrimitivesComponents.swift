// Copyright (c). Gem Wallet. All rights reserved.

import Components
import struct Gemstone.GemWalletRow
import GemstonePrimitives
import Primitives
import Style
import SwiftUI

public extension GemWalletRow {
    var image: Image {
        placeholder.image
    }

    var badgeImage: Image? {
        showsWatchBadge ? Images.Wallets.watch : nil
    }

    var listItem: ListItemModel {
        ListItemModel(title: name, titleExtra: subtitle.text, imageStyle: .asset(assetImage: avatarImage))
    }

    var avatarImage: AssetImage {
        AssetImage(
            type: .text(name),
            imageURL: imageUrl.map { ImageSource($0).url },
            placeholder: image,
            chainPlaceholder: badgeImage,
        )
    }
}

extension GemWalletRow: @retroactive Identifiable {}

public extension GemWalletRow {
    var nameListItem: ListItemModel {
        ListItemModel(title: name, imageStyle: .asset(assetImage: avatarImage))
    }
}
