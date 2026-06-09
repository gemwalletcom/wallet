// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Primitives
import Style
import SwiftUI

struct SupportMessageBubbleViewModel: Identifiable {
    private let message: SupportMessage
    private let retryAction: (SupportMessage) -> Void
    private let imageAction: (SupportImagePreviewRequest) -> Void
    private let displayURLResolver: (SupportMessageImage) -> URL?

    init(
        message: SupportMessage,
        retryAction: @escaping (SupportMessage) -> Void,
        imageAction: @escaping (SupportImagePreviewRequest) -> Void,
        displayURL: @escaping (SupportMessageImage) -> URL?,
    ) {
        self.message = message
        self.retryAction = retryAction
        self.imageAction = imageAction
        self.displayURLResolver = displayURL
    }

    var id: String { message.id }
    var content: String { message.content.trimmingCharacters(in: .whitespacesAndNewlines) }
    var hasContent: Bool { content.isNotEmpty }
    var hasImages: Bool { message.images.isNotEmpty }
    var images: [SupportMessageImage] { message.images }
    var isSending: Bool { message.status == .sending }
    var isFailed: Bool { message.status == .failed }

    var palette: Palette {
        switch message.sender {
        case .user:
            Palette(text: Colors.whiteSolid, background: Colors.blue, secondary: Colors.whiteSolid, link: Colors.whiteSolid)
        case .agent:
            Palette(text: Colors.black, background: Colors.white, secondary: Colors.secondaryText, link: Colors.blue)
        }
    }

    var time: String { message.createdAt.formatted(date: .omitted, time: .shortened) }

    var status: Status {
        switch message.status {
        case .sending: .sending
        case .sent: .sent(time: time)
        case .failed: .failed
        }
    }

    func retry() {
        retryAction(message)
    }

    func onImageTap(_ image: SupportMessageImage) {
        switch message.status {
        case .sending:
            break
        case .failed:
            retry()
        case .sent:
            imageAction(SupportImagePreviewRequest(images: message.images, selectedId: image.id))
        }
    }

    func displayURL(for image: SupportMessageImage) -> URL? {
        displayURLResolver(image)
    }
}

// MARK: - Types

extension SupportMessageBubbleViewModel {
    struct Palette {
        let text: Color
        let background: Color
        let secondary: Color
        let link: Color
    }

    enum Status {
        case sending
        case sent(time: String)
        case failed
    }
}
