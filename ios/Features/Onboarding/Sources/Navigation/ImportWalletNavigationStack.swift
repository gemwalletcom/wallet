// Copyright (c). Gem Wallet. All rights reserved.

import GemstoneServices
import Primitives
import PrimitivesComponents
import SwiftUI

public struct ImportWalletNavigationStack: View {
    @State private var model: ImportWalletViewModel
    @State private var navigationPath = NavigationPath()

    public init(model: ImportWalletViewModel) {
        _model = State(initialValue: model)
    }

    public var body: some View {
        NavigationStack(path: $navigationPath) {
            rootScene
                .toolbarDismissItem(type: .close, placement: .topBarLeading)
                .navigationBarTitleDisplayMode(.inline)
                .navigationDestination(for: ImportWalletType.self) { type in
                    ImportWalletScene(model: model.importWalletModel(type: type))
                }
                .navigationDestination(for: Scenes.ImportWalletType.self) { _ in
                    importWalletTypeScene
                }
        }
    }

    @ViewBuilder
    private var rootScene: some View {
        if model.isAcceptTermsCompleted {
            importWalletTypeScene
        } else {
            AcceptTermsScene(model: AcceptTermsViewModel(preferences: model.preferences, onNext: { navigate(to: .importWalletType) }))
        }
    }

    private var importWalletTypeScene: some View {
        ImportWalletTypeScene(model: model.importWalletTypeModel())
    }
}

// MARK: - Actions

extension ImportWalletNavigationStack {
    func navigate(to route: ImportWalletRoute) {
        switch route {
        case .importWalletType: navigationPath.append(Scenes.ImportWalletType())
        }
    }
}
