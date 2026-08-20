// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Foundation
import Primitives
import PrimitivesComponents
import Store
import SwiftUI

public enum NFTGridPosterBuilder {
    public static func item(from data: NFTData) -> GridPosterViewItem {
        if data.assets.count == 1, let asset = data.assets.first {
            return item(collection: data.collection, asset: asset)
        }
        return GridPosterViewItem(
            id: data.id,
            destination: Scenes.Collection(id: data.collection.id.identifier, name: data.collection.name),
            model: GridPosterViewModel(
                assetImage: AssetImage(type: .text(data.collection.name), imageURL: data.collection.images.preview.url.asURL),
                title: data.collection.name,
                count: data.assets.count,
                isVerified: data.collection.status == .verified,
            ),
        )
    }

    public static func item(collection: NFTCollection, asset: NFTAsset) -> GridPosterViewItem {
        GridPosterViewItem(
            id: asset.id.identifier,
            destination: Scenes.Collectible(assetData: NFTAssetData(collection: collection, asset: asset)),
            model: GridPosterViewModel(
                assetImage: AssetImage(type: .text(collection.name), imageURL: asset.images.preview.url.asURL),
                title: asset.name,
                isVerified: collection.status == .verified,
            ),
        )
    }
}
