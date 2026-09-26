// Copyright (c). Gem Wallet. All rights reserved.

import Components
import enum Gemstone.GemAcquireOption
import Localization
import Primitives
import PrimitivesComponents
import Style
import SwiftUI

public struct GetAssetScene: View {
    private let asset: Asset
    private let options: [GemAcquireOption]
    private let onSelect: (GemAcquireOption) -> Void

    public init(
        asset: Asset,
        options: [GemAcquireOption],
        onSelect: @escaping (GemAcquireOption) -> Void,
    ) {
        self.asset = asset
        self.options = options
        self.onSelect = onSelect
    }

    public var body: some View {
        List {
            ForEach(options) { option($0) }
        }
        .contentMargins(.top, .scene.top, for: .scrollContent)
        .listStyle(.insetGrouped)
        .scrollDisabled(true)
        .navigationTitle(Localized.Asset.getAsset(asset.symbol))
        .navigationBarTitleDisplayMode(.inline)
        .toolbar {
            ToolbarItem(placement: .principal) {
                Text(Localized.Asset.getAsset(asset.symbol))
                    .font(.headline)
                    .fontWeight(.medium)
            }
        }
    }
}

// MARK: - UI Components

extension GetAssetScene {
    private func option(_ option: GemAcquireOption) -> some View {
        NavigationCustomLink(
            with: row(
                title: option.title,
                subtitle: option.subtitle,
                image: option.image,
                color: option.color,
            ),
            action: { onSelect(option) },
        )
        .listRowInsets(.assetListRowInsets)
    }

    private func row(
        title: String,
        subtitle: String,
        image: Image,
        color: Color,
    ) -> some View {
        ListItemFlexibleView(
            left: { icon(image: image, color: color) },
            primary: {
                VStack(alignment: .leading, spacing: .tiny) {
                    Text(title)
                        .textStyle(.body.weight(.medium))
                        .lineLimit(1)

                    Text(subtitle)
                        .textStyle(.calloutSecondary)
                        .lineLimit(1)
                }
            },
            secondary: {
                EmptyView()
            },
        )
    }

    private func icon(image: Image, color: Color) -> some View {
        image
            .font(.system(size: 18, weight: .semibold))
            .foregroundStyle(Colors.whiteSolid)
            .frame(width: .list.settings, height: .list.settings)
            .background(color)
            .clipShape(RoundedRectangle(cornerRadius: .space8))
    }
}
