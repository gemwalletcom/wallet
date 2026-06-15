// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Primitives
import Style
import SwiftUI

struct SupportMessageBubble: View {
    let model: SupportMessageBubbleViewModel

    private enum Constants {
        static let imageWidth: CGFloat = 240
        static let imageHeight: CGFloat = 180
        static let maxWidth: CGFloat = 300
    }

    var body: some View {
        VStack(alignment: .leading, spacing: .tiny) {
            if model.hasImages {
                imagesView
            }
            if model.hasContent {
                textBubble
            }
        }
        .frame(maxWidth: Constants.maxWidth, alignment: .leading)
    }

    private var textBubble: some View {
        (Text(.init(model.content)) + timeSpacer)
            .font(.body)
            .foregroundStyle(model.palette.text)
            .tint(model.palette.link)
            .overlay(alignment: .bottomTrailing) {
                statusView
            }
            .padding(.vertical, .small)
            .padding(.horizontal, .space12)
            .background(model.palette.background)
            .clipShape(RoundedRectangle(cornerRadius: .space16))
            .contextMenu(.copy(value: model.content))
    }

    private var timeSpacer: Text {
        Text(verbatim: "  \(model.time)")
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
