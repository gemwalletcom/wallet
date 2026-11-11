// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Localization
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

            if model.hasPayloadFields {
                Section {
                    ForEach(Array(model.primaryPayloadFields.enumerated()), id: \.offset) {
                        GemListRowView(row: $0.element, onSelectAddress: model.onSelectPayloadAddress)
                    }

                    if model.secondaryPayloadFields.isNotEmpty {
                        NavigationCustomLink(with: ListItemView(model: model.payloadDetailsListItem)) {
                            model.onViewPayloadDetails()
                        }
                    }
                }
            } else if let messageText = model.messageText {
                Section(Localized.SignMessage.message) {
                    Text(messageText)
                }
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
