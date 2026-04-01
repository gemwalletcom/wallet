// Copyright (c). Gem Wallet. All rights reserved.

import Localization
import Primitives
import SwiftUI

enum ExportWalletDestination: Hashable {
    case words([String])
    case privateKey(String)
}

public struct ExportWalletNavigationStack: View {
    private let flow: ExportWalletType
    @State private var navigationPath: NavigationPath = .init()

    public init(flow: ExportWalletType) {
        self.flow = flow
    }

    public var body: some View {
        NavigationStack(path: $navigationPath) {
            Group {
                switch flow {
                case .words:
                    SecurityReminderScene(
                        model: SecurityReminderViewModelDefault(
                            title: Localized.Common.secretPhrase,
                            onNext: onNext,
                        ),
                    )
                case let .privateKey(key):
                    ShowSecretDataScene(model: ShowPrivateKeyViewModel(text: key))
                }
            }
            .toolbarDismissItem(type: .close, placement: .topBarLeading)
            .navigationBarTitleDisplayMode(.inline)
            .navigationDestination(for: ExportWalletDestination.self) {
                switch $0 {
                case let .words(words):
                    ShowSecretDataScene(model: ShowSecretPhraseViewModel(words: words))
                case let .privateKey(key):
                    ShowSecretDataScene(model: ShowPrivateKeyViewModel(text: key))
                }
            }
        }
    }
}

extension ExportWalletNavigationStack {
    private func onNext() {
        switch flow {
        case let .words(words):
            navigationPath.append(ExportWalletDestination.words(words))
        case let .privateKey(key):
            navigationPath.append(ExportWalletDestination.privateKey(key))
        }
    }
}
