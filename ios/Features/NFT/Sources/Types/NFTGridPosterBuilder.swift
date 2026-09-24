// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Foundation
import struct Gemstone.GemNftEntry
import enum Gemstone.GemNftItem
import struct Gemstone.GemNftRow
import GemstonePrimitives
import Primitives
import PrimitivesComponents
import Store
import SwiftUI

public enum NFTGridPosterBuilder {
    public static func items(_ entries: [GemNftEntry]) -> [GridPosterViewItem] {
        entries.map { item($0.item, $0.row) }
    }

    private static func item(_ item: GemNftItem, _ row: GemNftRow) -> GridPosterViewItem {
        switch item {
        case let .collection(data): collection(data.toPrimitives(), row)
        case let .asset(data): asset(data.toPrimitives(), row)
        }
    }

    private static func collection(_ data: NFTData, _ row: GemNftRow) -> GridPosterViewItem {
        GridPosterViewItem(
            id: row.id,
            destination: Scenes.Collection(id: data.collection.id.identifier),
            model: model(row),
        )
    }

    private static func asset(_ data: NFTAssetData, _ row: GemNftRow) -> GridPosterViewItem {
        GridPosterViewItem(
            id: row.id,
            destination: Scenes.Collectible(assetData: data),
            model: model(row),
        )
    }

    private static func model(_ row: GemNftRow) -> GridPosterViewModel {
        GridPosterViewModel(
            assetImage: AssetImage(type: .text(row.title), imageURL: row.imageUrl.asURL),
            title: row.title,
            countText: row.countText,
            isVerified: row.isVerified,
        )
    }
}
