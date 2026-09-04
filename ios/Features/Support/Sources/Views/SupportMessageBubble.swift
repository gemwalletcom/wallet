// Copyright (c). Gem Wallet. All rights reserved.

import Components
import struct Gemstone.SupportMessageLink
import Primitives
import Style
import SwiftUI

struct SupportMessageBubble: View {
    let model: SupportMessageBubbleViewModel

    @Environment(\.openURL) private var openURL

    private enum Constants {
        static let imageWidth: CGFloat = 240
        static let imageHeight: CGFloat = 180
        static let maxWidth: CGFloat = 300
    }

    var body: some View {
        HStack(alignment: .center, spacing: .small) {
            if model.isFailed {
                failedIndicator
            }
            messageView
        }
        .frame(maxWidth: Constants.maxWidth, alignment: model.alignment)
    }

    private var messageView: some View {
        VStack(alignment: model.alignment.horizontal, spacing: .tiny) {
            if model.hasImages {
                imagesView
            }
            if model.hasContent {
                textBubble
            }
        }
    }

    private var failedIndicator: some View {
        Image(systemName: SystemImage.errorOccurred)
            .font(.body)
            .foregroundStyle(Colors.red)
    }

    private var textBubble: some View {
        VStack(alignment: .leading, spacing: .zero) {
            if model.hasDisplayText {
                messageTextView
            }
            if model.hasLinks {
                linksView
                if !model.hasDisplayText {
                    HStack {
                        Spacer(minLength: .zero)
                        statusView
                    }
                    .padding(.horizontal, .space12)
                    .padding(.bottom, .small)
                }
            }
        }
        .background(model.palette.background)
        .clipShape(RoundedRectangle(cornerRadius: .space16))
        .contextMenu(.copy(value: model.content))
    }

    private var messageTextView: some View {
        (Text(.init(model.displayText)) + timeSpacer)
            .font(.body)
            .foregroundStyle(model.palette.text)
            .tint(model.palette.link)
            .overlay(alignment: .bottomTrailing) {
                statusView
            }
            .padding(.vertical, .small)
            .padding(.horizontal, .space12)
    }

    private var linksView: some View {
        VStack(spacing: .zero) {
            if model.hasDisplayText {
                linkDivider
            }
            ForEach(Array(model.links.enumerated()), id: \.offset) { index, link in
                if index > .zero {
                    linkDivider
                        .padding(.leading, .space12)
                }
                linkRow(link)
            }
        }
    }

    private var linkDivider: some View {
        Divider()
            .overlay(model.palette.secondary.opacity(.medium))
    }

    private func linkRow(_ link: SupportMessageLink) -> some View {
        Button {
            if let url = link.url.asURL {
                openURL(url)
            }
        } label: {
            HStack(alignment: .center, spacing: .small) {
                HStack(alignment: .top, spacing: .small) {
                    Image(systemName: SystemImage.textPageFill)
                        .font(.caption)
                        .foregroundStyle(model.palette.link)
                        .frame(size: .list.selected.image)
                    VStack(alignment: .leading, spacing: .space2) {
                        Text(link.title)
                            .font(.callout)
                            .foregroundStyle(model.palette.link)
                            .lineLimit(2)
                            .fixedSize(horizontal: false, vertical: true)
                            .multilineTextAlignment(.leading)
                        if let subtitle = link.subtitle {
                            Text(subtitle)
                                .font(.caption)
                                .foregroundStyle(model.palette.secondary)
                                .lineLimit(1)
                        }
                    }
                }
                .frame(maxWidth: .infinity, alignment: .topLeading)
                .layoutPriority(1)
                Image(systemName: SystemImage.chevronRight)
                    .font(.caption2)
                    .foregroundStyle(model.palette.secondary)
                    .frame(size: .list.selected.image)
            }
            .contentShape(Rectangle())
            .padding(.horizontal, .space12)
            .padding(.vertical, .small)
        }
        .buttonStyle(.plain)
    }

    private var timeSpacer: Text {
        Text(verbatim: "    \(model.time)")
            .font(.caption2)
            .foregroundStyle(Color.clear)
    }

    private var imagesView: some View {
        VStack(spacing: .tiny) {
            ForEach(model.images, id: \.id) { image in
                imageView(image)
            }
        }
    }

    private func imageView(_ image: SupportMessageImage) -> some View {
        Button {
            model.onImageTap(image)
        } label: {
            CachedAsyncImage(url: model.imageURL(for: image)) { loaded in
                loaded.resizable().scaledToFill()
            } placeholder: {
                ZStack {
                    Colors.grayLightFaded
                    if !model.isFailed {
                        ProgressView()
                    }
                }
            }
            .frame(width: Constants.imageWidth, height: Constants.imageHeight)
            .clipShape(RoundedRectangle(cornerRadius: .space12))
            .contentShape(RoundedRectangle(cornerRadius: .space12))
            .overlay(alignment: .bottomTrailing) {
                if !model.isSending {
                    timePill
                }
            }
        }
        .buttonStyle(.plain)
    }

    private var timePill: some View {
        Text(model.time)
            .font(.caption2)
            .foregroundStyle(Colors.whiteSolid)
            .padding(.horizontal, .small)
            .padding(.vertical, .tiny)
            .background(Colors.blackSolid.opacity(.medium))
            .clipShape(Capsule())
            .padding(.small)
    }

    @ViewBuilder
    private var statusView: some View {
        switch model.status {
        case .sending:
            ProgressView()
                .controlSize(.small)
                .tint(model.palette.secondary)
        case let .sent(time):
            Text(time)
                .font(.caption2)
                .foregroundStyle(model.palette.secondary)
        case .failed:
            Button(action: model.retry) {
                Image(systemName: SystemImage.refresh)
                    .font(.caption)
                    .foregroundStyle(model.palette.secondary)
            }
            .buttonStyle(.plain)
        }
    }
}
