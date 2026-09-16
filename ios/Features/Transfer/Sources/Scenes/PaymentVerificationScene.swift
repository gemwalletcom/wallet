// Copyright (c). Gem Wallet. All rights reserved.

import Components
import InfoSheet
import Style
import SwiftUI

public struct PaymentVerificationScene: View {
    @State private var model: PaymentVerificationSceneViewModel

    public init(model: PaymentVerificationSceneViewModel) {
        _model = State(wrappedValue: model)
    }

    public var body: some View {
        NavigationStack {
            WebView(url: model.url, messageHandler: model.messageHandler)
            .navigationTitle(model.title)
            .navigationBarTitleDisplayMode(.inline)
            .toolbarDismissItem(type: .close, placement: .topBarLeading)
            .toolbar {
                ToolbarItem(placement: .topBarTrailing) {
                    Button("", systemImage: SystemImage.info, action: model.onSelectInfo)
                }
            }
        }
        .sheet(item: $model.isPresentingInfoSheet) {
            InfoSheetScene(type: $0)
        }
    }
}
