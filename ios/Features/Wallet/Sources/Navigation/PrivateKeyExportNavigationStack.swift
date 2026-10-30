// Copyright (c). Gem Wallet. All rights reserved.

import enum Gemstone.GemSecretExport
import protocol Gemstone.GemWalletServiceProtocol
import Primitives
import PrimitivesComponents
import SwiftUI

struct PrivateKeyExportNavigationStack: View {
    @State private var model: PrivateKeyExportViewModel

    init(walletId: WalletId, export: GemSecretExport, service: any GemWalletServiceProtocol) {
        _model = State(initialValue: PrivateKeyExportViewModel(walletId: walletId, export: export, service: service))
    }

    var body: some View {
        NavigationStack(path: $model.navigationPath) {
            SecurityReminderScene(
                model: SecurityReminderViewModel(
                    title: model.title,
                    kind: .accountKey,
                    onNext: onContinue,
                ),
            )
            .toolbarDismissItem(type: .close, placement: .topBarLeading)
            .navigationBarTitleDisplayMode(.inline)
            .navigationDestination(for: PrivateKeyExportDestination.self) {
                switch $0 {
                case .accounts: PrivateKeyAccountsScene(model: model)
                case let .secret(secret): ShowSecretDataScene(secret: secret)
                }
            }
        }
        .alertSheet($model.isPresentingAlertMessage)
    }

    private func onContinue() {
        Task { await model.onContinue() }
    }
}
