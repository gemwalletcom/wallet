// Copyright (c). Gem Wallet. All rights reserved.

import Primitives
import PrimitivesTestKit
@testable import Support

extension SupportMessageBubbleViewModel {
    static func mock(
        message: SupportMessage = .mock(),
        retryAction: @escaping (SupportMessage) -> Void = { _ in },
        imageAction: @escaping (SupportMessageImage) -> Void = { _ in },
    ) -> SupportMessageBubbleViewModel {
        SupportMessageBubbleViewModel(message: message, retryAction: retryAction, imageAction: imageAction)
    }
}
