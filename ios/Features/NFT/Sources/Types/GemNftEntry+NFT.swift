// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Foundation
import struct Gemstone.GemNftEntry
import GemstonePrimitives
import Localization
import Primitives
import PrimitivesComponents

public extension GemNftEntry {
    var destination: any Hashable & Sendable {
        switch item {
        case let .collection(data): Scenes.Collection(id: data.toPrimitives().collection.id.identifier)
        case let .asset(data): Scenes.Collectible.assetData(data.toPrimitives())
        }
    }

    var posterModel: GridPosterViewModel {
        GridPosterViewModel(
            assetImage: AssetImage(type: .text(row.title), imageURL: row.imageUrl.asURL),
            title: row.title,
            countText: row.countText,
            isVerified: row.isVerified,
        )
    }
}

