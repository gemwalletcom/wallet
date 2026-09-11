// Copyright (c). Gem Wallet. All rights reserved.

import enum Gemstone.GemWalletSecret
import Localization
import Primitives
import SwiftUI

enum ExportWalletDestination: Hashable {
    case words([String])
    case privateKey(String)
}

public struct ExportWalletNavigationStack: View {
    private let flow: GemWalletSecret
    @State private var navigationPath: NavigationPath = .init()

    public init(flow: GemWalletSecret) {
        self.flow = flow
    }

    public var body: some View {
        NavigationStack(path: $navigationPath) {
            let title = switch flow {
            case .words: Localized.Common.secretPhrase
            case .privateKey: Localized.Common.privateKey
            }
            SecurityReminderScene(
                model: SecurityReminderViewModel(
                    title: title,
                    onNext: onNext,
                ),
            )
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
