// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import func Gemstone.parseSupportMessageDisplayContent
import struct Gemstone.SupportMessageDisplayContent
import struct Gemstone.SupportMessageLink
import Primitives
import Style
import SwiftUI

struct SupportMessageBubbleViewModel: Identifiable {
    private let message: SupportMessage
    private let displayContent: SupportMessageDisplayContent
    private let retryAction: (SupportMessage) -> Void
    private let imageAction: (SupportMessageImage) -> Void

    init(
        message: SupportMessage,
        retryAction: @escaping (SupportMessage) -> Void,
        imageAction: @escaping (SupportMessageImage) -> Void,
    ) {
        self.message = message
        self.displayContent = parseSupportMessageDisplayContent(markdown: message.content)
        self.retryAction = retryAction
        self.imageAction = imageAction
    }

    var id: String { message.id }
    var content: String { message.content.trim() }
    var displayText: String { displayContent.text }
    var links: [SupportMessageLink] { displayContent.links }
    var hasContent: Bool { hasDisplayText || hasLinks }
    var hasDisplayText: Bool { displayText.isNotEmpty }
    var hasLinks: Bool { links.isNotEmpty }
    var hasImages: Bool { message.images.isNotEmpty }
    var images: [SupportMessageImage] { message.images }
    var isSending: Bool { message.status == .sending }
    var isFailed: Bool { message.status == .failed }

    var palette: Palette {
        switch message.sender {
        case .user: Palette(text: Colors.whiteSolid, background: Colors.blue, secondary: Colors.whiteSolid, link: Colors.whiteSolid)
        case .agent: Palette(text: Colors.black, background: Colors.white, secondary: Colors.secondaryText, link: Colors.blue)
        }
    }

    var alignment: Alignment {
        switch message.sender {
        case .user: .trailing
        case .agent: .leading
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

    func imageURL(for image: SupportMessageImage) -> URL? {
        image.url.asURL
    }

    func onImageTap(_ image: SupportMessageImage) {
        imageAction(image)
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
