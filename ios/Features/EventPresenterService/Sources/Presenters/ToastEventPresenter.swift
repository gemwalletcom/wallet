// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Style
import SwiftUI

@Observable
public final class ToastEventPresenter: Sendable {
    @MainActor
    public var toastMessage: ToastMessage?

    public init() {}

    @MainActor
    public func present(_ event: ToastEvent) {
        toastMessage = ToastEventMessageFactory.makeToastMessage(for: event)
    }
}
