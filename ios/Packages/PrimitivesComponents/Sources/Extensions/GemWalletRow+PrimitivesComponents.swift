// Copyright (c). Gem Wallet. All rights reserved.

import Components
import enum Gemstone.GemWalletPlaceholder
import struct Gemstone.GemWalletRow
import GemstonePrimitives
import Primitives
import Style
import SwiftUI

extension GemWalletPlaceholder {
    public var image: Image {
        switch self {
        case .multicoin: Images.Logo.logo
        case let .chain(chain): ChainImage(chain: Primitives.Chain(core: chain)).image
        }
    }
}

extension GemWalletRow {
    public var image: Image {
        placeholder.image
    }

    public var badgeImage: Image? {
        showsWatchBadge ? Images.Wallets.watch : nil
    }

    public var listItem: ListItemModel {
        ListItemModel(title: name, titleExtra: subtitle.text, imageStyle: .asset(assetImage: avatarImage))
    }

    public var avatarImage: AssetImage {
        AssetImage(
            type: .text(name),
            imageURL: imageUrl.map { ImageSource($0).url },
            placeholder: image,
            chainPlaceholder: badgeImage,
        )
    }
}

extension GemWalletRow: @retroactive Identifiable {}

extension GemWalletRow: @retroactive SimpleListItemViewable {
    public var title: String {
        name
    }

    public var assetImage: AssetImage {
        avatarImage
    }
}
