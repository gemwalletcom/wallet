// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import protocol Gemstone.GemStreamServiceProtocol
import GemstonePrimitivesTestKit
import Preferences
import PreferencesTestKit
import StreamService
import SupportChatService

public extension StreamEventService {
    static func mock(
        service: any GemStreamServiceProtocol = GemStreamServiceMock(),
        typing: SupportTypingState = SupportTypingState(),
        preferences: Preferences = .mock(),
    ) -> StreamEventService {
        StreamEventService(service: service, typing: typing, preferences: preferences)
    }
}
