// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Foundation
import struct Gemstone.GemCopy
import enum Gemstone.GemInfoTopic
import enum Gemstone.GemListRow
import enum Gemstone.GemListRowIcon
import enum Gemstone.GemListRowTitle
import enum Gemstone.GemNoticeKind
import struct Gemstone.GemSocialLink
import enum Gemstone.GemUrlTarget
import enum Gemstone.GemValueTone
import GemstonePrimitives
import Localization
import Primitives
import Style

struct AddressCardModel {
    let address: String
    let copyModel: CopyTypeViewModel
}

enum GemListRowItem {
    case notice(title: String, message: String?, kind: GemNoticeKind)
    case listItem(ListItemModel)
    case rate(title: String, direct: String, inverse: String)
    case provider(ListItemModel, contract: String?)
    case picker(ListItemModel, title: GemListRowTitle)
    case toggle(label: String, title: GemListRowTitle, isOn: Bool, imageStyle: ListItemImageStyle?)
    case page(ListItemModel, url: URL)
    case explorerPage(ListItemModel, context: ExplorerContextData)
    case external(ListItemModel, url: URL)
    case network(title: String, subtitle: String, image: AssetImage)
    case app(ListItemModel, website: URL?)
    case wallet(ListItemModel, context: ExplorerContextData)
    case memo(ListItemModel, copy: String?)
    case icon(AssetImage)
    case address(AddressCardModel)
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
        case let .ranked(title, amount, rank):
            .listItem(
                ListItemModel(
                    title: title.text,
                    titleTag: " #\(rank) ",
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
                image: AssetIdViewModel(assetId: Chain(core: chain).assetId).networkAssetImage,
            )
        case let .app(name, iconUrl, websiteUrl):
            .app(
                ListItemModel(
                    title: Localized.WalletConnect.app,
                    subtitle: name,
                    imageStyle: .list(assetImage: iconUrl.map { AssetImage(imageURL: URL(string: $0)) }),
                ),
                website: websiteUrl.flatMap { URL(string: $0) },
            )
        case let .wallet(wallet, copy, explorer):
            .wallet(
                ListItemModel(title: Localized.Common.wallet, subtitle: wallet.name, imageStyle: .list(assetImage: wallet.avatarImage)),
                context: ExplorerContextData(copyValue: copy.copyValue, explorerLink: explorer.toPrimitives()),
            )
        case let .memo(value, copy):
            .memo(ListItemModel(title: Localized.Transfer.memo, subtitle: value), copy: copy)
        case let .link(title, value, icon):
            .listItem(listItem(title: title, value: value, icon: icon))
        case let .picker(title, value, icon):
            .picker(listItem(title: title, value: value.text, icon: icon), title: title)
        case let .toggle(title, value, icon, isOn):
            .toggle(label: toggleLabel(title: title, value: value), title: title, isOn: isOn, imageStyle: icon.imageStyle)
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
        case let .identifier(title, copy, explorer):
            identifierItem(title: title, copy: copy, explorer: explorer?.toPrimitives())
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

    private func subtitleStyle(_ tone: GemValueTone) -> TextStyle {
        switch tone {
        case .plain: ListItemModel.StyleDefaults.subtitleStyle
        case .neutral, .positive, .warning, .negative: TextStyle(font: .callout, color: tone.color)
        }
    }

    private func infoAction(_ topic: GemInfoTopic?, onInfo: ((GemInfoTopic) -> Void)?) -> VoidAction {
        topic.flatMap { topic in onInfo.map { onInfo in { onInfo(topic) } } }
    }

    private func toggleLabel(title: GemListRowTitle, value: String?) -> String {
        switch title {
        case .authentication: value.map { Localized.Settings.enableValue($0) } ?? title.text
        default: title.text
        }
    }

    private func listItem(title: GemListRowTitle, value: String?, icon: GemListRowIcon) -> ListItemModel {
        ListItemModel(title: title.text, subtitle: value, imageStyle: icon.imageStyle)
    }

    private func identifierItem(title: GemListRowTitle, copy: GemCopy, explorer: BlockExplorerLink?) -> GemListRowItem {
        let model = ListItemModel(title: title.text, subtitle: copy.display)
        guard let explorer else { return .memo(model, copy: copy.value) }
        return .explorerPage(model, context: ExplorerContextData(copyValue: copy.copyValue, explorerLink: explorer))
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
