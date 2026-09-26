// Copyright (c). Gem Wallet. All rights reserved.

import Components
import struct Gemstone.GemSupportMessageRow
import struct Gemstone.SupportMessageLink
import GemstonePrimitives
import Primitives
import PrimitivesComponents
import Style
import SwiftUI

struct SupportMessageBubble: View {
    private let row: GemSupportMessageRow
    private let message: SupportMessage
    private let onRetry: (SupportMessage) -> Void
    private let onImage: (SupportMessageImage) -> Void

    @Environment(\.openURL) private var openURL

    init(row: GemSupportMessageRow, onRetry: @escaping (SupportMessage) -> Void, onImage: @escaping (SupportMessageImage) -> Void) {
        self.row = row
        message = row.message.toPrimitives()
        self.onRetry = onRetry
        self.onImage = onImage
    }

    private enum Constants {
        static let imageWidth: CGFloat = 240
        static let imageHeight: CGFloat = 180
        static let maxWidth: CGFloat = 300
    }

    var body: some View {
        HStack(alignment: .center, spacing: .small) {
            if isFailed {
                failedIndicator
            }
            messageView
        }
        .frame(maxWidth: Constants.maxWidth, alignment: row.side.alignment)
    }

    private var messageView: some View {
        VStack(alignment: row.side.alignment.horizontal, spacing: .tiny) {
            if message.images.isNotEmpty {
                imagesView
            }
            if hasContent {
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
            if hasDisplayText {
                messageTextView
            }
            if hasLinks {
                linksView
                if !hasDisplayText {
                    HStack {
                        Spacer(minLength: .zero)
                        statusView
                    }
                    .padding(.horizontal, .space12)
                    .padding(.bottom, .small)
                }
            }
        }
        .background(row.side.palette.background)
        .clipShape(RoundedRectangle(cornerRadius: .space16))
        .contextMenu(.copy(value: message.content.trim()))
    }

    private var messageTextView: some View {
        (Text(.init(row.content.text)) + timeSpacer)
            .font(.body)
            .foregroundStyle(row.side.palette.text)
            .tint(row.side.palette.link)
            .overlay(alignment: .bottomTrailing) {
                statusView
            }
            .padding(.vertical, .small)
            .padding(.horizontal, .space12)
    }

    private var linksView: some View {
        VStack(spacing: .zero) {
            if hasDisplayText {
                linkDivider
            }
            ForEach(Array(row.content.links.enumerated()), id: \.offset) { index, link in
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
            .overlay(row.side.palette.secondary.opacity(.medium))
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
                        .foregroundStyle(row.side.palette.link)
                        .frame(size: .list.selected.image)
                    VStack(alignment: .leading, spacing: .space2) {
                        Text(link.title)
                            .font(.callout)
                            .foregroundStyle(row.side.palette.link)
                            .lineLimit(2)
                            .fixedSize(horizontal: false, vertical: true)
                            .multilineTextAlignment(.leading)
                        if let subtitle = link.subtitle {
                            Text(subtitle)
                                .font(.caption)
                                .foregroundStyle(row.side.palette.secondary)
                                .lineLimit(1)
                        }
                    }
                }
                .frame(maxWidth: .infinity, alignment: .topLeading)
                .layoutPriority(1)
                Image(systemName: SystemImage.chevronRight)
                    .font(.caption2)
                    .foregroundStyle(row.side.palette.secondary)
                    .frame(size: .list.selected.image)
            }
            .contentShape(Rectangle())
            .padding(.horizontal, .space12)
            .padding(.vertical, .small)
        }
        .buttonStyle(.plain)
    }

    private var timeSpacer: Text {
        Text(verbatim: "    \(time)")
            .font(.caption2)
            .foregroundStyle(Color.clear)
    }

    private var imagesView: some View {
        VStack(spacing: .tiny) {
            ForEach(message.images, id: \.id) { image in
                imageView(image)
            }
        }
    }

    private func imageView(_ image: SupportMessageImage) -> some View {
        Button {
            onImage(image)
        } label: {
            CachedAsyncImage(url: image.url.asURL) { loaded in
                loaded.resizable().scaledToFill()
            } placeholder: {
                ZStack {
                    Colors.grayLightFaded
                    if !isFailed {
                        ProgressView()
                    }
                }
            }
            .frame(width: Constants.imageWidth, height: Constants.imageHeight)
            .clipShape(RoundedRectangle(cornerRadius: .space12))
            .contentShape(RoundedRectangle(cornerRadius: .space12))
            .overlay(alignment: .bottomTrailing) {
                if !isSending {
                    timePill
                }
            }
        }
        .buttonStyle(.plain)
    }

    private var time: String {
        message.createdAt.formatted(date: .omitted, time: .shortened)
    }

    private var hasDisplayText: Bool {
        row.content.text.isNotEmpty
    }

    private var hasLinks: Bool {
        row.content.links.isNotEmpty
    }

    private var hasContent: Bool {
        hasDisplayText || hasLinks
    }

    private var isSending: Bool {
        row.outcome == .sending
    }

    private var isFailed: Bool {
        if case .failed = row.outcome {
            true
        } else {
            false
        }
    }

    private var timePill: some View {
        Text(time)
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
        switch row.outcome {
        case .sending:
            ProgressView()
                .controlSize(.small)
                .tint(row.side.palette.secondary)
        case .sent:
            Text(time)
                .font(.caption2)
                .foregroundStyle(row.side.palette.secondary)
        case let .failed(canRetry):
            if canRetry {
                Button(action: { onRetry(message) }) {
                    Image(systemName: SystemImage.refresh)
                        .font(.caption)
                        .foregroundStyle(row.side.palette.secondary)
                }
                .buttonStyle(.plain)
            } else {
                Image(systemName: SystemImage.exclamationmarkTriangle)
                    .font(.caption)
                    .foregroundStyle(row.side.palette.secondary)
            }
        }
    }
}
