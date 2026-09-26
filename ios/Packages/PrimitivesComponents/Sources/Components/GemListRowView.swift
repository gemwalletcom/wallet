// Copyright (c). Gem Wallet. All rights reserved.

import Components
import enum Gemstone.GemInfoTopic
import enum Gemstone.GemListRow
import enum Gemstone.GemListRowTitle
import enum Gemstone.GemRowAction
import enum Gemstone.GemRowMenuItem
import Localization
import Primitives
import Style
import SwiftUI

public struct GemListRowView: View {
    @Environment(\.openURL) private var openURL

    @State private var presentation: GemListRowPresentationType?
    @State private var isRateInverse = false

    private let row: GemListRow
    private let onToggle: ((GemRowAction, Bool) -> Void)?
    private let onSelect: ((GemRowAction) -> Void)?
    private let onSelectAddress: ((String) -> Void)?
    private let onInfo: ((GemInfoTopic) -> Void)?
    private let onCopy: ((CopyTypeViewModel) -> Void)?

    public init(
        row: GemListRow,
        onToggle: ((GemRowAction, Bool) -> Void)? = nil,
        onSelect: ((GemRowAction) -> Void)? = nil,
        onSelectAddress: ((String) -> Void)? = nil,
        onInfo: ((GemInfoTopic) -> Void)? = nil,
        onCopy: ((CopyTypeViewModel) -> Void)? = nil,
    ) {
        self.row = row
        self.onToggle = onToggle
        self.onSelect = onSelect
        self.onSelectAddress = onSelectAddress
        self.onInfo = onInfo
        self.onCopy = onCopy
    }

    public var body: some View {
        content
    }

    @ViewBuilder
    private var content: some View {
        if case let .identifier(title, copy, _, address?, _) = row, let onSelectAddress {
            NavigationCustomLink(with: ListItemView(model: ListItemModel(title: title.text, subtitle: copy.display))) {
                onSelectAddress(address)
            }
        } else {
            itemContent
        }
    }

    @ViewBuilder
    private var itemContent: some View {
        switch row.item(onInfo: onInfo) {
        case let .notice(title, message, kind):
            switch kind {
            case .error, .warning: ListItemErrorView(errorTitle: message.map { _ in title }, errorImageColor: kind.color, error: AnyError(message ?? title))
            case .info: ListItemInfoView(title: title, description: message)
            }
        case let .listItem(model):
            ListItemView(model: model)
        case let .rate(title, direct, inverse):
            ListItemRotateView(title: title, subtitle: isRateInverse ? inverse : direct) { isRateInverse.toggle() }
        case let .provider(model, contract):
            if let contract, let onSelectAddress {
                NavigationCustomLink(with: ListItemView(model: model)) { onSelectAddress(contract) }
            } else {
                ListItemView(model: model)
            }
        case let .picker(model, action):
            NavigationCustomLink(with: ListItemView(model: model)) { onSelect?(action) }
        case let .toggle(label, action, isOn, imageStyle):
            if let imageStyle {
                ListItemToggleView(isOn: Binding(get: { isOn }, set: { onToggle?(action, $0) }), title: label, imageStyle: imageStyle)
            } else {
                Toggle(label, isOn: Binding(get: { isOn }, set: { onToggle?(action, $0) }))
                    .toggleStyle(AppToggleStyle())
            }
        case let .page(model, url):
            SafariNavigationLink(url: url) {
                ListItemView(model: model)
            }
        case let .explorerPage(model, url, menu):
            SafariNavigationLink(url: url) {
                ListItemView(model: model)
            }
            .contextMenu(contextMenu(menu))
            .safariSheet(url: isPresentingUrl)
        case let .external(model, url):
            NavigationCustomLink(with: ListItemView(model: model)) {
                openURL(url)
            }
        case let .network(title, subtitle, image):
            ListItemImageView(title: title, subtitle: subtitle, assetImage: image)
        case let .imageMenu(model, menu):
            ListItemImageView(model: model)
                .contextMenu(contextMenu(menu))
                .safariSheet(url: isPresentingUrl)
        case let .menu(model, menu):
            ListItemView(model: model)
                .contextMenu(contextMenu(menu))
                .safariSheet(url: isPresentingUrl)
        case let .social(links):
            SocialLinksView(links: links)
        case let .icon(assetImage):
            AssetImageView(assetImage: assetImage, size: .image.semiLarge)
                .frame(maxWidth: .infinity)
                .padding(.bottom, .small)
                .cleanListRow()
        case let .address(model):
            AddressRowView(model: model, onCopy: { onCopy?(model.copyModel) })
        case .loading:
            ListItemLoadingView()
        }
    }
}

extension GemListRowView {
    private func contextMenu(_ menu: [GemRowMenuItem]) -> [ContextMenuItemType] {
        menu.map { item in
            switch item {
            case let .copy(copy): .copy(value: copy.value)
            case let .open(title, url): .url(title: title.text, onOpen: { presentation = URL(string: url).map { .url($0) } })
            }
        }
    }

    private var isPresentingUrl: Binding<URL?> {
        Binding(
            get: {
                guard case let .url(url) = presentation else { return nil }
                return url
            },
            set: { presentation = $0.map { .url($0) } },
        )
    }
}
