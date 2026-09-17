// Copyright (c). Gem Wallet. All rights reserved.

import Components
import enum Gemstone.GemImage
import GemstonePrimitives
import Primitives

struct AssetListItemViewModel: Identifiable {
    private let list: AssetList
    init(list: AssetList) {
        self.list = list
    }

    var id: String {
        list.id
    }

    var listItem: ListItemModel {
        ListItemModel(title: name, subtitle: count, imageStyle: .settings(assetImage: image))
    }

    var name: String {
        list.name
    }

    var count: String {
        String(list.count)
    }

    var image: AssetImage {
        AssetImage(type: .text(list.name), imageURL: GemImage.assetList(listId: list.id).imageURL)
    }
}
