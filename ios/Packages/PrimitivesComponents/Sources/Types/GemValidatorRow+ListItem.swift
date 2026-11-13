// Copyright (c). Gem Wallet. All rights reserved.

import Components
import GemstonePrimitives
import Primitives
import struct Gemstone.GemValidatorRow

public extension GemValidatorRow {
    var listItem: ListItemModel {
        ListItemModel(title: name, subtitle: apr.text)
    }

    var explorerContext: ExplorerContextData? {
        let validator = validator.toPrimitives()
        return explorer.map {
            ExplorerContextData(copyValue: .address(value: validator.id, chain: validator.chain), explorerLink: $0.toPrimitives())
        }
    }

    var assetImage: AssetImage {
        if let image = provider?.image {
            return AssetImage(placeholder: image)
        }
        return AssetImage(type: .text(placeholder), imageURL: imageUrl.asURL)
    }
}
