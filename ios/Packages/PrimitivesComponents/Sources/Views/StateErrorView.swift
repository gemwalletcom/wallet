// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Localization
import Style
import SwiftUI

public struct StateErrorView: View {
    private let error: Error
    private let onRetry: @Sendable () async -> Void

    public init(error: Error, onRetry: @escaping @Sendable () async -> Void) {
        self.error = error
        self.onRetry = onRetry
    }

    public var body: some View {
        StateEmptyView(
            title: Localized.Errors.errorOccurred,
            description: error.localizedDescription,
            image: nil,
        ) {
            Button(Localized.Common.tryAgain) {
                Task { await onRetry() }
            }
            .buttonStyle(.blue())
        }
    }
}
