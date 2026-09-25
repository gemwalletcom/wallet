// Copyright (c). Gem Wallet. All rights reserved.

import protocol Gemstone.GemNotificationsServiceProtocol
import protocol Gemstone.GemSupportServiceProtocol
import GemstonePrimitivesTestKit
import GemstoneServices
import Support

public extension SupportChatSceneViewModel {
    static func mock(
        service: any GemSupportServiceProtocol = GemSupportServiceMock(),
        notifications: any GemNotificationsServiceProtocol = GemNotificationsServiceMock(),
        typing: ObservableSupportTyping = ObservableSupportTyping(),
    ) -> SupportChatSceneViewModel {
        SupportChatSceneViewModel(service: service, notifications: notifications, typing: typing)
    }
}
