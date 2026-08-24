// Copyright (c). Gem Wallet. All rights reserved.

import Payments
import SwiftUI

struct PaymentNavigationView: View {
    @State private var model: PaymentSceneViewModel

    init(model: PaymentSceneViewModel) {
        _model = State(initialValue: model)
    }

    var body: some View {
        PaymentScene(model: model)
            .sheet(item: $model.isPresentingSheet) {
                switch $0 {
                case let .dataCollection(url):
                    PaymentDataCollectionScene(url: url, onComplete: model.onCompleteDataCollection)
                case .quotes:
                    PaymentQuotesSelectionScene(model: model)
                }
            }
    }
}
