// Copyright (c). Gem Wallet. All rights reserved.

import Components
import enum Gemstone.GemInfoTopic
import enum Gemstone.GemListRow
import enum Gemstone.GemListRowTitle
import Localization
import Primitives
import Style
import SwiftUI

public struct GemListRowView: View {
    @Environment(\.openURL) private var openURL

    @State private var presentation: GemListRowPresentationType?
    @State private var isRateInverse = false

    private let row: GemListRow
    private let onToggle: ((GemListRowTitle, Bool) -> Void)?
    private let onSelect: ((GemListRowTitle) -> Void)?
    private let onSelectAddress: ((String) -> Void)?
    private let onInfo: ((GemInfoTopic) -> Void)?

    public init(
        row: GemListRow,
        onToggle: ((GemListRowTitle, Bool) -> Void)? = nil,
        onSelect: ((GemListRowTitle) -> Void)? = nil,
        onSelectAddress: ((String) -> Void)? = nil,
        onInfo: ((GemInfoTopic) -> Void)? = nil,
    ) {
        self.row = row
        self.onToggle = onToggle
        self.onSelect = onSelect
        self.onSelectAddress = onSelectAddress
        self.onInfo = onInfo
    }

    public var body: some View {
        content
    }

    @ViewBuilder
    private var content: some View {
        if case let .identifier(.contract, copy, _) = row, let onSelectAddress {
            NavigationCustomLink(with: ListItemView(model: ListItemModel(title: GemListRowTitle.contract.text, subtitle: copy.display))) {
                onSelectAddress(copy.value)
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
        case let .picker(model, title):
            NavigationCustomLink(with: ListItemView(model: model)) { onSelect?(title) }
        case let .toggle(label, title, isOn, imageStyle):
            if let imageStyle {
                ListItemToggleView(isOn: Binding(get: { isOn }, set: { onToggle?(title, $0) }), title: label, imageStyle: imageStyle)
            } else {
                Toggle(label, isOn: Binding(get: { isOn }, set: { onToggle?(title, $0) }))
                    .toggleStyle(AppToggleStyle())
            }
        case let .page(model, url):
            SafariNavigationLink(url: url) {
                ListItemView(model: model)
            }
        case let .explorerPage(model, context):
            SafariNavigationLink(url: context.explorerLink.url) {
                ListItemView(model: model)
            }
            .explorerContext(context)
        case let .external(model, url):
            NavigationCustomLink(with: ListItemView(model: model)) {
                openURL(url)
            }
        case let .network(title, subtitle, image):
            ListItemImageView(title: title, subtitle: subtitle, assetImage: image)
        case let .app(model, website):
            ListItemImageView(model: model)
                .contextMenu(website.map { url in [.url(title: Localized.Settings.website, onOpen: { presentation = .url(url) })] } ?? [])
                .safariSheet(url: isPresentingUrl)
        case let .wallet(model, context):
            ListItemImageView(model: model)
                .explorerContext(context)
        case let .memo(model, copy):
            ListItemView(model: model)
                .contextMenu(copy.map { [.copy(value: $0)] } ?? [])
        case let .social(links):
            SocialLinksView(model: SocialLinksViewModel(links: links))
        case let .icon(assetImage):
            AssetImageView(assetImage: assetImage, size: .image.semiLarge)
                .frame(maxWidth: .infinity)
                .padding(.bottom, .small)
                .cleanListRow()
        case .loading:
            ListItemLoadingView()
        }
    }
}

extension GemListRowView {
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

extension GemListRow: ItemModelProvidable {
    public var itemModel: GemListRow {
        self
    }
}
