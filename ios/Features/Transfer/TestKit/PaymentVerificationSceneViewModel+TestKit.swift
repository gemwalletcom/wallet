// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Transfer

public extension PaymentVerificationSceneViewModel {
    @MainActor
    static func mock(onComplete: @escaping () -> Void = {}, onError: @escaping () -> Void = {}) -> PaymentVerificationSceneViewModel {
        PaymentVerificationSceneViewModel(url: URL(string: "https://pay.walletconnect.com/collect")!, onComplete: onComplete, onError: onError)
    }
}
