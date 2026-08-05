// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Localization
import Onboarding
import Payments
import PriceService
import Primitives
import Style
import SwiftUI
import WalletConnector

struct RootScene: View {
    @Environment(\.scenePhase) private var scenePhase
    @State private var model: RootSceneViewModel

    init(model: RootSceneViewModel) {
        _model = State(initialValue: model)
    }

    var body: some View {
        VStack {
            if let currentWallet = model.currentWallet {
                MainTabView(model: .init(wallet: currentWallet))
                    .alertSheet($model.updateVersionAlertMessage)
            } else {
                OnboardingScene(
                    isPresentingCreateWalletSheet: $model.isPresentingCreateWalletSheet,
                    isPresentingImportWalletSheet: $model.isPresentingImportWalletSheet,
                )
            }
        }
        .onOpenURL { url in
            Task {
                await model.handleOpenUrl(url)
            }
        }
        .sheet(item: $model.isPresentingConnectorSheet, onDismiss: model.walletConnectorPresenter.onSheetDismiss) { type in
            WalletConnectorNavigationStack(
                type: type,
                presenter: model.walletConnectorPresenter,
            )
        }
        .sheet(item: $model.isPresentingPaymentSheet, onDismiss: model.paymentSheetPresenter.onSheetDismiss) { type in
            PaymentNavigationStack(
                type: type,
                presenter: model.paymentSheetPresenter,
            )
        }
        .sheet(isPresented: $model.isPresentingCreateWalletSheet) {
            CreateWalletNavigationStack(
                model: CreateWalletModel(
                    walletService: model.walletService,
                    walletSessionService: model.walletSessionService,
                    avatarService: model.avatarService,
                    onComplete: model.dismissCreateWallet,
                ),
            )
        }
        .sheet(isPresented: $model.isPresentingImportWalletSheet) {
            ImportWalletNavigationStack(
                model: ImportWalletViewModel(
                    walletService: model.walletService,
                    walletSessionService: model.walletSessionService,
                    avatarService: model.avatarService,
                    nameService: model.nameService,
                    onComplete: model.dismissImportWallet,
                ),
            )
        }
        .alert(
            Localized.WalletConnect.brandName,
            presenting: $model.isPresentingConnectorError,
            actions: { _ in
                Button(
                    Localized.Common.done,
                    role: .none,
                    action: {},
                )
            },
            message: {
                Text(model.isPresentingConnectorError.valueOrEmpty)
            },
        )
        .taskOnce(model.setup)
        .lockManaged(by: model.lockManager)
        .onChange(
            of: model.currentWallet,
            initial: true,
            model.onChangeWallet,
        )
        .toast(
            isPresenting: $model.isPresentingConnectorBar,
            message: ToastMessage(
                title: "\(Localized.WalletConnect.brandName)...",
                image: SystemImage.network,
            ),
            offsetY: -model.toastOffset,
        )
        .toast(message: $model.isPresentingToastMessage, offsetY: -model.toastOffset)
        .onChange(of: scenePhase, model.onScenePhaseChanged)
        .onChange(of: model.observablePreferences.isPerpetualEnabled, model.onPerpetualEnabledChanged)
    }
}
