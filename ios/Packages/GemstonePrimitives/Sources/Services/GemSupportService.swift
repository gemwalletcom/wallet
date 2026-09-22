// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import protocol Gemstone.GemSupportServiceProtocol
import Primitives

public extension GemSupportServiceProtocol {
    func sendMessage(_ content: SupportMessageContent) async throws {
        switch content {
        case let .text(text):
            try await sendText(content: text)
        case let .image(data):
            try await sendImage(image: data)
        }
    }

    func retryMessage(_ message: SupportMessage) async throws {
        try await retryMessage(message: message.toGem())
    }
}
