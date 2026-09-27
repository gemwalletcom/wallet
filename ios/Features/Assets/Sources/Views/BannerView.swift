// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Foundation
import enum Gemstone.GemBannerButton
import struct Gemstone.GemBannerContent
import enum Gemstone.GemBannerDestination
import struct Gemstone.GemBannerKey
import struct Gemstone.GemBannerRow
import Primitives
import PrimitivesComponents
import Style
import SwiftUI

public struct BannerView: View {
    private let row: GemBannerRow
    private let onDestination: (GemBannerDestination) -> Void
    private let onButton: (GemBannerButton) -> Void
    private let onClose: (GemBannerKey) -> Void

    public init(
        row: GemBannerRow,
        onDestination: @escaping (GemBannerDestination) -> Void,
        onButton: @escaping (GemBannerButton) -> Void,
        onClose: @escaping (GemBannerKey) -> Void,
    ) {
        self.row = row
        self.onDestination = onDestination
        self.onButton = onButton
        self.onClose = onClose
    }

    private var content: GemBannerContent {
        row.content
    }

    public var body: some View {
        ZStack(alignment: .topTrailing) {
            switch content.style {
            case .list: listView
            case .welcome: bannerView
            }

            if content.canClose {
                closeButton
                    .padding([.top, .trailing], .medium)
            }
        }
    }
}

// MARK: - Private Views

private extension BannerView {
    private var listView: some View {
        Button(
            action: { content.destination.map(onDestination) },
            label: {
                HStack(spacing: .zero) {
                    ListItemView(model: ListItemModel(title: content.title?.text, titleExtra: content.description?.text, imageStyle: content.icon?.imageStyle))

                    Spacer(minLength: content.canClose ? .extraLarge : .zero)
                }
            },
        )
        .buttonStyle(.listStyleColor(glassEffect: .disabled))
    }

    private var bannerView: some View {
        VStack(spacing: .medium) {
            if let icon = content.icon, let image = icon.image {
                AssetImageView(assetImage: image, size: icon.imageSize)
            }

            VStack(spacing: .small) {
                if let title = content.title?.text {
                    Text(title)
                        .textStyle(TextStyle(font: .body, color: .primary, fontWeight: .semibold))
                }

                if let subtitle = content.description?.text {
                    Text(subtitle)
                        .textStyle(.bodySecondary)
                }
            }
            .multilineTextAlignment(.center)

            HStack(spacing: .medium) {
                ForEach(content.buttons, id: \.self) { button in
                    Button {
                        onButton(button)
                    } label: {
                        Text(button.title)
                    }
                    .frame(maxWidth: .infinity)
                    .buttonStyle(button.style)
                }
            }
        }
        .padding()
        .frame(maxWidth: .infinity)
    }

    private var closeButton: some View {
        Button {
            onClose(row.key)
        } label: {
            Images.System.xmark
                .resizable()
                .frame(size: .small)
                .symbolRenderingMode(.hierarchical)
                .foregroundStyle(Colors.gray)
                .padding(.small)
                .liquidGlass { _ in
                    ListButton(
                        image: Images.System.xmarkCircle,
                        action: { onClose(row.key) },
                    )
                    .foregroundStyle(Colors.gray)
                }
        }
        .buttonStyle(.borderless)
    }
}
