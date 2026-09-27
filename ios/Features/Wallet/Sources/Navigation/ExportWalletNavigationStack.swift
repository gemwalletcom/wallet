// Copyright (c). Gem Wallet. All rights reserved.

import enum Gemstone.GemWalletSecret
import Primitives
import PrimitivesComponents
import SwiftUI

struct ExportWalletNavigationStack: View {
    private let flow: GemWalletSecret
    @State private var navigationPath: NavigationPath = .init()

    init(flow: GemWalletSecret) {
        self.flow = flow
    }

    var body: some View {
        NavigationStack(path: $navigationPath) {
            SecurityReminderScene(
                model: SecurityReminderViewModel(
                    title: flow.title,
                    onNext: onNext,
                ),
            )
            .toolbarDismissItem(type: .close, placement: .topBarLeading)
            .navigationBarTitleDisplayMode(.inline)
            .navigationDestination(for: GemWalletSecret.self) {
                ShowSecretDataScene(secret: $0)
            }
        }
    }
}

extension ExportWalletNavigationStack {
    private func onNext() {
        navigationPath.append(flow)
    }
}
