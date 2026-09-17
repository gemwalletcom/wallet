// Copyright (c). Gem Wallet. All rights reserved.

import Components
import enum Gemstone.GemListRow
import Primitives
import Style
import SwiftUI

public struct GemListRowView: View {
    @State private var isPresentingCopyToast = false

    private let row: GemListRow

    public init(row: GemListRow) {
        self.row = row
    }

    public var body: some View {
        content
    }

    @ViewBuilder
    private var content: some View {
        switch row.item {
        case let .listItem(model):
            ListItemView(model: model)
        case let .link(model, url):
            SafariNavigationLink(url: url) {
                ListItemView(model: model)
            }
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
