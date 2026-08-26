// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import protocol Gemstone.GemStreamServiceProtocol
import GemstonePrimitives
import Preferences
import Primitives
import GemstoneServices

public struct StreamEventService: Sendable {
    private let service: any GemStreamServiceProtocol
    private let typing: SupportTypingState
    private let preferences: Preferences
    private let decoder = JSONDateDecoder.standard

    public init(
        service: any GemStreamServiceProtocol,
        typing: SupportTypingState,
        preferences: Preferences,
    ) {
        self.service = service
        self.typing = typing
        self.preferences = preferences
    }

    public func handle(_ data: Data) async {
        do {
            let event = try decoder.decode(StreamEvent.self, from: data)
            try await service.handle(event: String(decoding: data, as: UTF8.self), currency: Currency(id: preferences.currency).json())
            if case let .support(supportEvent) = event {
                await handleSupport(supportEvent)
            }
        } catch {
            debugLog("stream event handler error: \(error)")
        }
    }

    private func handleSupport(_ event: SupportStreamEvent) async {
        switch event {
        case let .message(message):
            switch message.sender {
            case .user: break
            case .agent: await typing.clear()
            }
        case let .typing(payload):
            await typing.update(payload)
        }
    }
}
