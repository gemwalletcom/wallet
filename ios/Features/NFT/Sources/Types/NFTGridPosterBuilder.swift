// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Foundation
import enum Gemstone.GemNftItem
import GemstonePrimitives
import Primitives
import PrimitivesComponents
import Store
import SwiftUI

public enum NFTGridPosterBuilder {
    public static func item(_ item: GemNftItem) -> GridPosterViewItem {
        switch item {
        case let .collection(data): collection(data.map())
        case let .asset(data): asset(data.map())
        }
    }

    private static func collection(_ data: NFTData) -> GridPosterViewItem {
        GridPosterViewItem(
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

    private static func asset(_ data: NFTAssetData) -> GridPosterViewItem {
        GridPosterViewItem(
            id: data.asset.id.identifier,
            destination: Scenes.Collectible(assetData: data),
            model: GridPosterViewModel(
                assetImage: AssetImage(type: .text(data.collection.name), imageURL: data.asset.images.preview.url.asURL),
                title: data.asset.name,
                isVerified: data.collection.status == .verified,
            ),
        )
    }
}
