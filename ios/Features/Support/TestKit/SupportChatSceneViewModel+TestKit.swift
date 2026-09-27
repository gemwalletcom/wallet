// Copyright (c). Gem Wallet. All rights reserved.

import protocol Gemstone.GemSupportServiceProtocol
import GemstonePrimitivesTestKit
import GemstoneServices
import Support

public extension SupportChatSceneViewModel {
    static func mock(
        service: any GemSupportServiceProtocol = GemSupportServiceMock(),
        typing: ObservableSupportTyping = ObservableSupportTyping(),
    ) -> SupportChatSceneViewModel {
        SupportChatSceneViewModel(service: service, typing: typing)
    }
}
