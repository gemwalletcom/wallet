// Copyright (c). Gem Wallet. All rights reserved.

import Components
import struct Gemstone.GemValidatorRow
import PrimitivesComponents

public extension GemValidatorRow {
    var listItem: ListItemModel {
        ListItemModel(title: name, subtitle: apr.text)
    }

    var assetImage: AssetImage {
        if let image = provider?.image {
            return AssetImage(placeholder: image)
        }
        return AssetImage(type: .text(placeholder), imageURL: imageUrl.asURL)
    }
}
