// Copyright (c). Gem Wallet. All rights reserved.

import Components
import enum Gemstone.GemInfoTopic
import enum Gemstone.GemListRow
import enum Gemstone.GemListRowTitle
import Primitives
import Style
import SwiftUI

public struct GemListRowView: View {
    @Environment(\.openURL) private var openURL

    @State private var isPresentingCopyToast = false

    private let row: GemListRow
    private let onToggle: ((GemListRowTitle, Bool) -> Void)?
    private let onSelect: ((GemListRowTitle) -> Void)?
    private let onInfo: ((GemInfoTopic) -> Void)?

    public init(
        row: GemListRow,
        onToggle: ((GemListRowTitle, Bool) -> Void)? = nil,
        onSelect: ((GemListRowTitle) -> Void)? = nil,
        onInfo: ((GemInfoTopic) -> Void)? = nil,
    ) {
        self.row = row
        self.onToggle = onToggle
        self.onSelect = onSelect
        self.onInfo = onInfo
    }

    public var body: some View {
        content
    }

    @ViewBuilder
    private var content: some View {
        switch row.item(onInfo: onInfo) {
        case let .listItem(model):
            ListItemView(model: model)
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
        case let .external(model, url):
            NavigationCustomLink(with: ListItemView(model: model)) {
                openURL(url)
            }
        case let .social(links):
            SocialLinksView(model: SocialLinksViewModel(links: links))
        case let .icon(assetImage):
            AssetImageView(assetImage: assetImage, size: .image.semiLarge)
                .frame(maxWidth: .infinity)
                .padding(.bottom, .small)
                .cleanListRow()
        case let .address(model):
            AddressCardView(model: model, action: { isPresentingCopyToast = true })
                .cleanListRow()
                .copyToast(model: model.copyModel, isPresenting: $isPresentingCopyToast)
        case .loading:
            ListItemLoadingView()
        }
    }
}

extension GemListRow: ItemModelProvidable {
    public var itemModel: GemListRow {
        self
    }
}
