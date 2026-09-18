// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Foundation
import enum Gemstone.GemInfoTopic
import enum Gemstone.GemListRow
import enum Gemstone.GemListRowIcon
import enum Gemstone.GemListRowTitle
import enum Gemstone.GemUrlTarget
import struct Gemstone.GemSocialLink
import GemstonePrimitives
import Localization
import Primitives

struct AddressCardModel {
    let address: String
    let copyModel: CopyTypeViewModel
}

enum GemListRowItem {
    case listItem(ListItemModel)
    case picker(ListItemModel, title: GemListRowTitle)
    case toggle(label: String, title: GemListRowTitle, isOn: Bool, imageStyle: ListItemImageStyle?)
    case page(ListItemModel, url: URL)
    case external(ListItemModel, url: URL)
    case icon(AssetImage)
    case address(AddressCardModel)
    case social([GemSocialLink])
    case loading
}

extension GemListRow {
    func item(onInfo: ((GemInfoTopic) -> Void)?) -> GemListRowItem {
        switch self {
        case let .text(title, value):
            .listItem(ListItemModel(title: title.text, subtitle: value))
        case let .amount(title, amount, info):
            .listItem(ListItemModel(title: title.text, subtitle: amount.text(), infoAction: info.flatMap { topic in onInfo.map { onInfo in { onInfo(topic) } } }))
        case let .link(title, value, icon):
            .listItem(listItem(title: title, value: value, icon: icon))
        case let .picker(title, value, icon):
            .picker(listItem(title: title, value: value, icon: icon), title: title)
        case let .toggle(title, value, icon, isOn):
            .toggle(label: toggleLabel(title: title, value: value), title: title, isOn: isOn, imageStyle: .settings(assetImage: icon.assetImage))
        case let .url(title, value, icon, url, target):
            urlItem(title: title, value: value, icon: icon, url: url, target: target)
        case let .social(links):
            .social(links)
        case let .error(error):
            .listItem(ListItemModel(title: GemListRowTitle.error.text, subtitle: error.localizedDescription))
        case let .explorer(name, url):
            .page(ListItemModel(title: Localized.Transaction.viewOn(name)), url: URL(string: url) ?? BlockExplorerLink(name: name, link: url).url)
        case let .icon(chain):
            .icon(AssetIdViewModel(assetId: Chain(core: chain).assetId).assetImage)
        case let .address(address, copy):
            .address(AddressCardModel(address: address, copyModel: copy.copyModel))
        case .loading:
            .loading
        }
    }

    private func toggleLabel(title: GemListRowTitle, value: String?) -> String {
        switch title {
        case .authentication: value.map { Localized.Settings.enableValue($0) } ?? title.text
        default: title.text
        }
    }

    private func listItem(title: GemListRowTitle, value: String?, icon: GemListRowIcon) -> ListItemModel {
        ListItemModel(title: title.text, subtitle: value, imageStyle: .settings(assetImage: icon.assetImage))
    }

    private func urlItem(title: GemListRowTitle, value: String?, icon: GemListRowIcon, url: String, target: GemUrlTarget) -> GemListRowItem {
        let model = listItem(title: title, value: value, icon: icon)
        guard let url = URL(string: url) else { return .listItem(model) }
        switch target {
        case .inApp: return .page(model, url: url)
        case .external: return .external(model, url: url)
        }
    }
}
