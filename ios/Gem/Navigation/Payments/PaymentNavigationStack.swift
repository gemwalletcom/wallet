// Copyright (c). Gem Wallet. All rights reserved.

import Payments
import Primitives
import SigningRequestService
import Style
import SwiftUI
import Transfer
import WalletConnector

struct PaymentNavigationStack: View {
    @Environment(\.viewModelFactory) private var viewModelFactory

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
                case let .confirm(data):
                    ConfirmTransferNavigationView(
                        model: viewModelFactory.confirmTransferScene(
                            wallet: data.payload.wallet,
                            data: data.payload.transferData,
                            confirmTransferDelegate: data.delegate,
                            simulation: data.payload.simulation,
                            onComplete: { presenter.complete(type: type) },
                        ),
                    )
                case let .signMessage(data):
                    SignMessageScene(
                        model: viewModelFactory.signMessageScene(
                            payload: data.payload,
                            confirmTransferDelegate: data.delegate,
                        ),
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
