// Copyright (c). Gem Wallet. All rights reserved.

import Style
import SwiftUI

public struct PaymentNavigationStack: View {
    private let type: PaymentSheetType
    private let presenter: PaymentSheetPresenter

    public init(
        type: PaymentSheetType,
        presenter: PaymentSheetPresenter,
    ) {
        self.type = type
        self.presenter = presenter
    }

    public var body: some View {
        NavigationStack {
            Group {
                switch type {
                case let .quotes(data):
                    PaymentQuotesScene(
                        model: PaymentQuotesSceneViewModel(
                            request: data.payload,
                            confirmTransferDelegate: data.delegate,
                        ),
                        onComplete: { presenter.complete(type: type) },
                    )
                case let .dataCollection(data):
                    PaymentDataCollectionScene(
                        callback: data,
                        onComplete: { presenter.complete(type: type) },
                    )
                }
            }
            .interactiveDismissDisabled(true)
            .navigationBarTitleDisplayMode(.inline)
            .toolbar {
                ToolbarItem(placement: .topBarLeading) {
                    Button("", systemImage: SystemImage.xmark) {
                        presenter.cancelSheet(type: type)
                    }
                }
            }
        }
    }
}
