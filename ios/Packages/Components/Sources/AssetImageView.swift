// Copyright (c). Gem Wallet. All rights reserved.

import Style
import SwiftUI

public struct AssetImageView: View {
    @Environment(\.displayScale) private var scale

    private let size: CGFloat
    private let assetImage: AssetImage
    private var cornerRadius: CGFloat { style?.cornerRadius ?? size / 2 }
    private let style: Style?

    public init(
        assetImage: AssetImage,
        size: CGFloat = .image.asset,
        style: Style? = nil,
    ) {
        self.assetImage = assetImage
        self.size = max(1, size)
        self.style = style
    }

    public var body: some View {
        CachedAsyncImage(
            url: assetImage.imageURL,
            scale: scale,
            content: {
                $0.resizable().animation(.default)
            },
            placeholder: {
                placeholderView
            },
        )
        .frame(width: size, height: size)
        .cornerRadius(cornerRadius)
        .ifLet(style?.fontWeight) { view, weight in
            view.fontWeight(weight)
        }
        .ifLet(style?.foregroundColor) { view, color in
            view.foregroundStyle(color)
        }
        .assetBadge(assetImage.chainPlaceholder, size: size)
    }

    @ViewBuilder
    private var placeholderView: some View {
        if let placeholderImage = assetImage.placeholder {
            placeholderImage
                .resizable()
                .scaledToFit()
                .cornerRadius(cornerRadius)
        } else if let tokenType = assetImage.type {
            GeometryReader { geo in
                let diameter = min(geo.size.width, geo.size.height)
                ZStack {
                    Circle()
                        .ifElse(tokenType.count == 1) { view in
                            view.foregroundStyle(.clear)
                        } elseContent: { view in
                            view.foregroundStyle(.tertiary)
                        }

                    Text(tokenType.uppercased())
                        .ifElse(tokenType.count == 1, ifContent: { view in
                            view
                                .font(.system(size: diameter, weight: .semibold))
                        }, elseContent: { view in
                            view
                                .font(.system(size: diameter * 0.35, weight: .semibold))
                                .padding(.horizontal, .space6)
                        })
                        .minimumScaleFactor(0.4)
                        .lineLimit(1)
                        .multilineTextAlignment(.center)
                        .foregroundStyle(.primary)
                }
            }
        } else {
            Rectangle()
                .foregroundStyle(.tertiary)
                .cornerRadius(cornerRadius)
        }
    }
}

// MARK: - Badge Overlay

public extension View {
    func assetBadge(_ badgeImage: Image?, size: CGFloat = .image.asset) -> some View {
        let overlaySize = size <= 48 ? size / 2.6 : size / 3
        let overlayPadding: CGFloat = 2
        let overlayOffset = (size / 2) - (overlaySize / 2)

        return self.overlay {
            if let badgeImage {
                badgeImage
                    .resizable()
                    .aspectRatio(contentMode: .fit)
                    .frame(width: overlaySize, height: overlaySize)
                    .padding(overlayPadding)
                    .background(Colors.listStyleColor)
                    .clipShape(Circle())
                    .offset(x: overlayOffset + (size / overlaySize), y: overlayOffset + (size / overlaySize))
            }
        }
    }
}

// MARK: - Style

public extension AssetImageView {
    struct Style: Sendable, Equatable {
        public let foregroundColor: Color?
        public let cornerRadius: CGFloat?
        public let fontWeight: Font.Weight?

        public init(
            foregroundColor: Color? = nil,
            cornerRadius: CGFloat? = nil,
            fontWeight: Font.Weight? = nil,
        ) {
            self.foregroundColor = foregroundColor
            self.cornerRadius = cornerRadius
            self.fontWeight = fontWeight
        }
    }
}

#Preview {
    Group {
        AssetImageView(
            assetImage: AssetImage(
                type: "SPL",
                imageURL: URL(string: "https://example.com/token.png"),
                placeholder: Image(systemName: "bitcoinsign.circle"),
                chainPlaceholder: Image(systemName: "bolt.circle.fill"),
            ),
            size: .image.medium,
        )

        AssetImageView(
            assetImage: AssetImage(
                type: "MIGRAINE",
                imageURL: nil,
                placeholder: nil,
                chainPlaceholder: Image(systemName: "bolt.circle.fill"),
            ),
            size: .image.medium,
        )
        .preferredColorScheme(.dark)

        AssetImageView(
            assetImage: AssetImage(
                type: Emoji.WalletAvatar.gem.rawValue,
                imageURL: nil,
                placeholder: nil,
                chainPlaceholder: Image(systemName: "bolt.circle.fill"),
            ),
            size: .image.medium,
        )
        .preferredColorScheme(.dark)
    }
}
