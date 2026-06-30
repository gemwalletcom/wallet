// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Primitives

struct AssetListItemViewModel: Identifiable {
    private let list: AssetList
    private let assetImageFormatter: AssetImageFormatter

    init(list: AssetList, assetImageFormatter: AssetImageFormatter = .shared) {
        self.list = list
        self.assetImageFormatter = assetImageFormatter
    }

    var id: String {
        list.id
    }

    var name: String {
        list.name
    }

    var count: String {
        String(list.count)
    }

    var image: AssetImage {
        AssetImage(type: list.name, imageURL: assetImageFormatter.getListUrl(for: list.id))
    }
}
