// Copyright (c). Gem Wallet. All rights reserved.

import Components
import struct Gemstone.GemAssetText
import Primitives
import Style
import SwiftUI

public struct AssetsCollectionView<Content: View>: View {
    private let models: [GemAssetText]
    private let content: (GemAssetText) -> Content

    public init(
        models: [GemAssetText],
        @ViewBuilder content: @escaping (GemAssetText) -> Content,
    ) {
        self.models = models
        self.content = content
    }

    public var body: some View {
        ScrollView(.horizontal, showsIndicators: false) {
            HStack(spacing: Spacing.small) {
                ForEach(models, id: \.asset.id) { model in
                    content(model)
                }
            }
        }
    }
}

public struct AssetChipView: View {
    private let model: GemAssetText

    public init(model: GemAssetText) {
        self.model = model
    }

    public var body: some View {
        HStack(spacing: .tiny) {
            AssetImageView(
                assetImage: model.assetImage,
                size: .list.image,
            )
            Text(model.asset.symbol)
                .textStyle(TextStyle(font: .body, color: .primary, fontWeight: .semibold))
        }
        .padding(.small)
        .background(Colors.listStyleColor, in: RoundedRectangle(cornerRadius: .list.image / 2 + .small))
    }
}
