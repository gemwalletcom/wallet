// Copyright (c). Gem Wallet. All rights reserved.

import Components
import SwiftUI

public struct PaymentVerificationScene: View {
    @State private var model: PaymentVerificationViewModel

    public init(model: PaymentVerificationViewModel) {
        _model = State(wrappedValue: model)
    }

    public var body: some View {
        NavigationStack {
            WebView(
                url: model.url,
                allowedHost: model.allowedHost,
                messageHandler: model.messageHandler,
            )
            .navigationBarTitleDisplayMode(.inline)
            .toolbarDismissItem(type: .close, placement: .topBarLeading)
        }
        .alertSheet($model.isPresentingAlertMessage)
    }
}
