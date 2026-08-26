// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Foundation

@Observable
public final class ToastPresenter: Sendable {
    @MainActor
    public var toastMessage: ToastMessage?

    public init() {}

    @MainActor
    public func present(_ message: ToastMessage?) {
        toastMessage = message
    }
}
