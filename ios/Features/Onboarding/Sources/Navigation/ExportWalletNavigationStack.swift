// Copyright (c). Gem Wallet. All rights reserved.

import Components
import func Gemstone.walletPrivateKeyChains
import GemstonePrimitives
import Localization
import Primitives
import PrimitivesComponents
import SwiftUI

enum ExportWalletDestination: Hashable {
    case words([String])
    case privateKey(chain: Chain, key: String)
    case chainReminder(Chain)
}

public struct ExportWalletNavigationStack: View {
    private let flow: ExportWalletFlow
    private let onExportPrivateKey: (Chain) async throws -> String
    private let chains: [Chain]
    @State private var navigationPath: NavigationPath = .init()
    @State private var networksModel: NetworkSelectorViewModel
    @State private var isPresentingAlertMessage: AlertMessage?

    public init(
        flow: ExportWalletFlow,
        onExportPrivateKey: @escaping (Chain) async throws -> String,
    ) {
        self.flow = flow
        self.onExportPrivateKey = onExportPrivateKey
        chains = if case let .privateKeyChains(wallet) = flow { walletPrivateKeyChains(wallet: wallet.map()).map { Chain(core: $0) } } else { [] }
        _networksModel = State(initialValue: NetworkSelectorViewModel(state: .data(.plain(chains)), title: Localized.Common.privateKey))
    }

    public var body: some View {
        NavigationStack(path: $navigationPath) {
            Group {
                switch flow {
                case let .words(words):
                    SecurityReminderScene(
                        model: SecurityReminderViewModel(
                            title: Localized.Common.secretPhrase,
                            onNext: { navigationPath.append(ExportWalletDestination.words(words)) },
                        ),
                    )
                case let .privateKey(chain, key):
                    reminder(chain: chain) {
                        navigationPath.append(ExportWalletDestination.privateKey(chain: chain, key: key))
                    }
                case .privateKeyChains:
                    if chains.count == 1, let chain = chains.first {
                        reminder(chain: chain) { onExport(chain: chain) }
                    } else {
                        SelectableListView(
                            model: $networksModel,
                            onFinishSelection: onSelectChains,
                            listContent: { ChainView(model: ChainViewModel(chain: $0)) },
                        )
                        .navigationTitle(networksModel.title)
                    }
                }
            }
            .toolbarDismissItem(type: .close, placement: .topBarLeading)
            .navigationBarTitleDisplayMode(.inline)
            .navigationDestination(for: ExportWalletDestination.self) {
                switch $0 {
                case let .words(words):
                    ShowSecretDataScene(model: ShowSecretPhraseViewModel(words: words))
                case let .privateKey(chain, key):
                    ShowSecretDataScene(model: ShowPrivateKeyViewModel(chain: chain, text: key))
                case let .chainReminder(chain):
                    reminder(chain: chain) { onExport(chain: chain) }
                }
            }
            .alertSheet($isPresentingAlertMessage)
        }
    }

    private func reminder(chain: Chain, onNext: @escaping () -> Void) -> some View {
        SecurityReminderScene(model: SecurityReminderViewModel(chain: chain, onNext: onNext))
    }
}

extension ExportWalletNavigationStack {
    private func onSelectChains(_ chains: [Chain]) {
        guard let chain = chains.first else { return }
        navigationPath.append(ExportWalletDestination.chainReminder(chain))
    }

    private func onExport(chain: Chain) {
        Task { @MainActor in
            do {
                let key = try await onExportPrivateKey(chain)
                navigationPath.append(ExportWalletDestination.privateKey(chain: chain, key: key))
            } catch {
                isPresentingAlertMessage = AlertMessage(error: error)
            }
        }
    }
}
