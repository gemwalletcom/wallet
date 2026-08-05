// Copyright (c). Gem Wallet. All rights reserved.

import Payments
import SwiftUI

struct PaymentNavigationStack: View {
    private let type: PaymentSheetType
    private let presenter: PaymentSheetPresenter

    init(
        type: PaymentSheetType,
        presenter: PaymentSheetPresenter,
    ) {
        self.type = type
        self.presenter = presenter
    }

    var body: some View {
        Group {
            switch type {
            case let .quotes(data):
                PaymentQuotesScene(
                    model: PaymentQuotesSceneViewModel(
                        request: data.payload,
                        confirmTransferDelegate: data.delegate,
                    ),
                    onComplete: complete,
                )
            case let .dataCollection(data):
                PaymentDataCollectionScene(callback: data, onComplete: complete)
            case let .confirm(data):
                SigningRequestSheetView(content: .transfer(data), onComplete: complete)
            case let .signMessage(data):
                SigningRequestSheetView(content: .signMessage(data), onComplete: complete)
            }
        }
        .signingRequestSheet(onCancel: { presenter.cancelSheet(type: type) })
    }

    private func complete() {
        presenter.complete(type: type)
    }
}
