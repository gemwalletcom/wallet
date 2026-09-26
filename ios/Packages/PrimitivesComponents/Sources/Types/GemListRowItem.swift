// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Foundation
import struct Gemstone.GemAssetIcon
import struct Gemstone.GemCopy
import enum Gemstone.GemInfoTopic
import enum Gemstone.GemListRow
import enum Gemstone.GemListRowIcon
import enum Gemstone.GemListRowTitle
import enum Gemstone.GemNoticeKind
import enum Gemstone.GemRowAction
import enum Gemstone.GemRowMenuItem
import struct Gemstone.GemSocialLink
import enum Gemstone.GemUrlTarget
import enum Gemstone.GemValueTone
import GemstonePrimitives
import Localization
import Primitives
import Style

struct AddressRowModel {
    let address: String
    let copy: GemCopy
}

enum GemListRowItem {
    case notice(title: String, message: String?, kind: GemNoticeKind)
    case listItem(ListItemModel)
    case rate(title: String, direct: String, inverse: String)
    case provider(ListItemModel, contract: String?)
    case picker(ListItemModel, action: GemRowAction)
    case toggle(label: String, action: GemRowAction, isOn: Bool, imageStyle: ListItemImageStyle?)
    case page(ListItemModel, url: URL)
    case explorerPage(ListItemModel, url: URL, menu: [GemRowMenuItem])
    case external(ListItemModel, url: URL)
    case network(title: String, subtitle: String, image: AssetImage)
    case imageMenu(ListItemModel, menu: [GemRowMenuItem])
    case menu(ListItemModel, menu: [GemRowMenuItem])
    case icon(AssetImage)
    case address(AddressRowModel)
    case social([GemSocialLink])
    case loading
}

extension GemListRow {
    func item(onInfo: ((GemInfoTopic) -> Void)?) -> GemListRowItem {
        switch self {
        case let .latency(title, titleSuffix, host, status):
            .listItem(status.listItem(title: title.text + titleSuffix, titleExtra: host))
        case let .notice(title, message, kind):
            .notice(title: title.text, message: message?.text, kind: kind)
        case let .text(title, value):
            .listItem(ListItemModel(title: title.text, subtitle: value))
        case let .provider(title, name, contract):
            .provider(ListItemModel(title: title.text, subtitle: name), contract: contract)
        case let .amount(title, amount, info):
            .listItem(ListItemModel(title: title.text, subtitle: amount.text(), subtitleStyle: subtitleStyle(amount.tone), infoAction: infoAction(info, onInfo: onInfo)))
        case let .rate(title, rate, inverse):
            if let inverse {
                .rate(title: title.text, direct: rate.text(formattedValue: rate.value.text()), inverse: inverse.text(formattedValue: inverse.value.text()))
            } else {
                .listItem(ListItemModel(title: title.text, subtitle: rate.text(formattedValue: rate.value.text())))
            }
        case let .action(title, value, info):
            .listItem(
                ListItemModel(
                    title: title.text,
                    titleStyle: info == nil ? ListItemModel.StyleDefaults.titleStyle : .bodySecondary,
                    subtitle: value?.text(),
                    infoAction: infoAction(info, onInfo: onInfo),
                ),
            )
        case let .quote(title, value, change):
            .listItem(
                ListItemModel(
                    title: title.text,
                    subtitle: value?.text(),
                    subtitleSuffix: change?.text(),
                    subtitleSuffixStyle: change.map { subtitleStyle($0.tone) } ?? ListItemModel.StyleDefaults.subtitleStyle,
                ),
            )
        case let .ranked(title, amount, tag):
            .listItem(
                ListItemModel(
                    title: title.text,
                    titleTag: " \(tag) ",
                    titleTagStyle: TextStyle(font: .system(.body), color: Colors.grayLight, background: Colors.grayVeryLight),
                    subtitle: amount.text(),
                ),
            )
        case let .allTime(title, value, date, change):
            .listItem(
                ListItemModel(
                    title: title.text,
                    titleExtra: TransactionDateFormatter(date: date).section,
                    subtitle: value.text(),
                    subtitleExtra: change.text(),
                    subtitleStyleExtra: TextStyle(font: .callout, color: change.tone.color),
                ),
            )
        case let .duration(title, parts, info, estimate):
            .listItem(
                ListItemModel(
                    title: title.text,
                    subtitle: estimate ? EstimatedConfirmationFormatter().string(parts: parts) : CountdownFormatter().string(parts: parts),
                    infoAction: infoAction(info, onInfo: onInfo),
                ),
            )
        case let .label(title, text, tone, info, progress):
            .listItem(
                ListItemModel(
                    title: title.text,
                    subtitle: text.text,
                    subtitleStyle: subtitleStyle(tone),
                    subtitleTagType: progress ? .progressView() : .none,
                    infoAction: infoAction(info, onInfo: onInfo),
                ),
            )
        case let .date(title, date):
            .listItem(ListItemModel(title: title.text, subtitle: TransactionDateFormatter(date: date).row))
        case let .network(title, chain, name):
            .network(
                title: title.text,
                subtitle: name,
                image: AssetImage(type: .text(.empty), placeholder: ChainImage(chain: Chain(core: chain)).image),
            )
        case let .app(title, name, iconUrl, menu):
            .imageMenu(
                ListItemModel(
                    title: title.text,
                    subtitle: name,
                    imageStyle: .list(assetImage: iconUrl.map { AssetImage(imageURL: URL(string: $0)) }),
                ),
                menu: menu,
            )
        case let .wallet(title, wallet, menu):
            .imageMenu(ListItemModel(title: title.text, subtitle: wallet.name, imageStyle: .list(assetImage: wallet.avatarImage)), menu: menu)
        case let .memo(title, value, menu):
            .menu(ListItemModel(title: title.text, subtitle: value), menu: menu)
        case let .link(title, value, icon, _):
            .listItem(listItem(title: title, value: value, icon: icon))
        case let .picker(title, value, icon, action):
            .picker(listItem(title: title, value: value.text, icon: icon), action: action)
        case let .toggle(label, icon, isOn, action):
            .toggle(label: label.text, action: action, isOn: isOn, imageStyle: icon.imageStyle)
        case let .url(title, value, icon, url, target):
            urlItem(title: title, value: value, icon: icon, url: url, target: target)
        case let .social(links):
            .social(links)
        case let .error(error):
            .notice(title: GemListRowTitle.error.text, message: error.localizedDescription, kind: .error)
        case let .lines(title, lines, info):
            .listItem(
                ListItemModel(
                    title: title.text,
                    subtitle: lines.first?.text,
                    subtitleExtra: lines.dropFirst().first?.text,
                    infoAction: infoAction(info, onInfo: onInfo),
                ),
            )
        case let .identifier(title, copy, explorer, _, menu):
            identifierItem(ListItemModel(title: title.text, subtitle: copy.display), explorer: explorer?.toPrimitives(), menu: menu)
        case let .explorer(title, url):
            URL(string: url).map { .page(ListItemModel(title: title.text), url: $0) } ?? .listItem(ListItemModel(title: title.text))
        case let .icon(icon, imageUrl):
            .icon(headerImage(icon: icon, imageUrl: imageUrl))
        case let .avatar(avatar):
            .icon(avatar.assetImage)
        case let .walletAvatar(imageUrl, placeholder):
            .icon(AssetImage(imageURL: imageUrl.map { ImageSource($0).url }, placeholder: placeholder.image))
        case let .address(address, copy):
            .address(AddressRowModel(address: address, copy: copy))
        case .loading:
            .loading
        }
    }

    private func subtitleStyle(_ tone: GemValueTone) -> TextStyle {
        switch tone {
        case .plain: ListItemModel.StyleDefaults.subtitleStyle
        case .neutral, .positive, .warning, .negative: TextStyle(font: .callout, color: tone.color)
        }
    }

    private func infoAction(_ topic: GemInfoTopic?, onInfo: ((GemInfoTopic) -> Void)?) -> VoidAction {
        topic.flatMap { topic in onInfo.map { onInfo in { onInfo(topic) } } }
    }

    private func listItem(title: GemListRowTitle, value: String?, icon: GemListRowIcon) -> ListItemModel {
        ListItemModel(title: title.text, subtitle: value, imageStyle: icon.imageStyle)
    }

    private func identifierItem(_ model: ListItemModel, explorer: BlockExplorerLink?, menu: [GemRowMenuItem]) -> GemListRowItem {
        guard let explorer else { return .menu(model, menu: menu) }
        return .explorerPage(model, url: explorer.url, menu: menu)
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

public extension GemListRow {
    func listItemModel(onInfo: ((GemInfoTopic) -> Void)? = nil) -> ListItemModel? {
        guard case let .listItem(model) = item(onInfo: onInfo) else { return nil }
        return model
    }
}

private extension GemListRow {
    func headerImage(icon: GemAssetIcon, imageUrl: String?) -> AssetImage {
        let assetImage = AssetImage(icon: icon)
        guard let imageUrl else { return assetImage }
        return AssetImage(imageURL: URL(string: imageUrl), placeholder: assetImage.placeholder)
    }
}
