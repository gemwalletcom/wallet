// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Style
import SwiftUI

public struct CurrencyInputValidationView: View {
    @Binding private var text: String

    private let error: (any Error)?
    private let config: CurrencyInputConfigurable
    private let infoAction: (any Error) -> (() -> Void)?

    public init(
        text: Binding<String>,
        error: (any Error)?,
        config: CurrencyInputConfigurable,
        infoAction: @escaping (any Error) -> (() -> Void)? = { _ in nil },
    ) {
        _text = text
        self.error = error
        self.config = config
        self.infoAction = infoAction
    }

    public var body: some View {
        VStack(spacing: .small) {
            CurrencyInputView(
                text: $text,
                config: config,
            )

            if let error, !(error is SilentValidationError) {
                HStack {
                    if let action = infoAction(error) {
                        InfoButton(action: action)
                    }
                    Text(.init(error.localizedDescription))
                        .multilineTextAlignment(.center)
                        .textStyle(TextStyle(font: .footnote, color: Colors.red))
                        .transition(.opacity)
                }
            }
        }
    }
}
