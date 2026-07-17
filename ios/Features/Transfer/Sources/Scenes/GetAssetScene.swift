// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Localization
import Primitives
import Style
import SwiftUI

public struct GetAssetScene: View {
    private let asset: Asset
    private let onSelect: (GetAssetAction) -> Void

    public init(
        asset: Asset,
        onSelect: @escaping (GetAssetAction) -> Void,
    ) {
        self.asset = asset
        self.onSelect = onSelect
    }

    public var body: some View {
        List {
            option(
                action: .buy,
                title: Localized.Wallet.buy,
                subtitle: Localized.Wallet.payWithCardOrBank,
                image: Images.System.plus,
                color: Colors.blue,
            )
            option(
                action: .swap,
                title: Localized.Wallet.swap,
                subtitle: Localized.Wallet.fromYourWalletAssets,
                image: Images.System.arrowSwap,
                color: Colors.green,
            )
            option(
                action: .receive,
                title: Localized.Wallet.receive,
                subtitle: Localized.Wallet.transferFromAnotherWallet,
                image: Image(systemName: "arrow.down"),
                color: Color.purple,
            )
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
    private func option(
        action: GetAssetAction,
        title: String,
        subtitle: String,
        image: Image,
        color: Color,
    ) -> some View {
        NavigationCustomLink(
            with: row(
                title: title,
                subtitle: subtitle,
                image: image,
                color: color,
            ),
            action: { onSelect(action) },
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
