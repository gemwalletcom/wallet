// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Primitives
import Style
import SwiftUI

struct SupportMessageBubbleViewModel: Identifiable {
    private let message: SupportMessage
    private let retryAction: (SupportMessage) -> Void

    init(message: SupportMessage, retryAction: @escaping (SupportMessage) -> Void) {
        self.message = message
        self.retryAction = retryAction
    }

    var id: String { message.id }
    var content: String { message.content }
    var hasContent: Bool { message.content.isNotEmpty }
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
