// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Primitives
import PrimitivesComponents
import Style
import SwiftUI

public struct AuthenticationScene: View {
    @Bindable var model: AuthenticationSceneViewModel
    private let onComplete: () -> Void

    public init(
        model: AuthenticationSceneViewModel,
        onComplete: @escaping () -> Void,
    ) {
        self.model = model
        self.onComplete = onComplete
    }

    public var body: some View {
        List {
            ListAssetHeaderView(model: model.appPreview, subtitleLayout: .vertical)

            Section {
                NavigationLink(value: Scenes.SelectWallet()) {
                    GemListRowView(row: model.walletListRow)
                }
                if let networkRow = model.networkRow {
                    GemListRowView(row: networkRow)
                }
            }

            if let messageText = model.messageText {
                SignMessagePayloadSection(
                    primaryRows: model.primaryPayloadFields,
                    secondaryRows: model.secondaryPayloadFields,
                    messageText: messageText,
                    onSelectAddress: model.onSelectPayloadAddress,
                    onViewDetails: model.onViewPayloadDetails,
                )
            }
        }
        .contentMargins(.top, .scene.top, for: .scrollContent)
        .listSectionSpacing(.compact)
        .safeAreaButton {
            StateButton(
                text: model.buttonTitle,
                state: model.buttonState,
                action: { model.onSign(onComplete: onComplete) },
            )
        }
        .navigationTitle(model.title)
        .navigationDestination(keyedBy: Scenes.SelectWallet.self) { _ in
            SelectWalletScene(model: $model.walletSelectorModel)
        }
        .alertSheet($model.isPresentingAlertMessage)
    }
}
