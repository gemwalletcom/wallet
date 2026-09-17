// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Foundation
import enum Gemstone.GemListRow
import enum Gemstone.GemListRowTitle
import GemstonePrimitives
import Localization
import Primitives

struct AddressCardModel {
    let address: String
    let copyModel: CopyTypeViewModel
}

enum GemListRowItem {
    case listItem(ListItemModel)
    case link(ListItemModel, url: URL)
    case icon(AssetImage)
    case address(AddressCardModel)
    case loading
}

extension GemListRow {
    var item: GemListRowItem {
        switch self {
        case let .text(title, value):
            .listItem(ListItemModel(title: title.text, subtitle: value))
        case let .amount(title, amount):
            .listItem(ListItemModel(title: title.text, subtitle: amount.text()))
        case let .error(error):
            .listItem(ListItemModel(title: GemListRowTitle.error.text, subtitle: error.localizedDescription))
        case let .explorer(name, url):
            .link(ListItemModel(title: Localized.Transaction.viewOn(name)), url: URL(string: url) ?? BlockExplorerLink(name: name, link: url).url)
        case let .icon(chain):
            .icon(AssetIdViewModel(assetId: Chain(core: chain).assetId).assetImage)
        case let .address(address, copy):
            .address(AddressCardModel(address: address, copyModel: copy.copyModel))
        case .loading:
            .loading
        }
    }
}
