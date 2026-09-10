// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Localization
import Primitives
import PrimitivesComponents
import Style
import SwiftUI

public struct SignMessageScene: View {
    @State private var model: SignMessageSceneViewModel
    private let onComplete: () -> Void

    public init(
        model: SignMessageSceneViewModel,
        onComplete: @escaping () -> Void,
    ) {
        _model = State(wrappedValue: model)
        self.onComplete = onComplete
    }

    public var body: some View {
        List {
            if let headerData = model.headerData {
                TransactionHeaderListItemView(headerType: .assetValue(headerData), showClearHeader: true)
            } else {
                ListAssetHeaderView(model: model.appPreview, subtitleLayout: .vertical)
            }

            Section {
                if model.headerData != nil {
                    ListItemImageView(
                        title: Localized.WalletConnect.app,
                        subtitle: model.appText,
                        assetImage: model.appAssetImage,
                    )
                }
                ListItemImageView(
                    title: Localized.Common.wallet,
                    subtitle: model.walletText,
                    assetImage: model.walletAssetImage,
                )
                ListItemImageView(
                    title: Localized.Transfer.network,
                    subtitle: model.networkText,
                    assetImage: model.networkAssetImage,
                )
            }

            if model.hasWarnings {
                Section {
                    SimulationWarningsContent(warnings: model.simulationWarnings)
                }
            }

            if model.payloadModel.hasFields {
                Section {
                    SimulationPayloadFieldsContent(
                        fields: model.payloadModel.primaryFields,
                        fieldViewModel: model.payloadModel.fieldViewModel(for:),
                        contextMenuItems: model.contextMenuItems(for:),
                    )

                    NavigationCustomLink(with: ListItemView(title: Localized.Common.details)) {
                        model.onViewPayloadDetails()
                    }
                }
            } else {
                Section(Localized.SignMessage.message) {
                    Text(model.messageText)
                }
            }
        }
        .contentMargins(.top, .scene.top, for: .scrollContent)
        .listSectionSpacing(.compact)
        .taskOnce { model.load() }
        .safeAreaButton {
            StateButton(
                text: model.buttonTitle,
                type: model.buttonType,
                action: sign,
            )
        }
        .navigationTitle(model.title)
        .alertSheet($model.isPresentingAlertMessage)
        .safariSheet(url: $model.isPresentingUrl)
        .sheet(isPresented: $model.isPresentingPayloadDetails) {
            if model.payloadModel.hasFields {
                NavigationStack {
                    SimulationPayloadDetailsScene(
                        primaryFields: model.payloadModel.primaryFields,
                        secondaryFields: model.payloadModel.secondaryFields,
                        fieldViewModel: model.payloadModel.fieldViewModel(for:),
                        contextMenuItems: model.contextMenuItems(for:),
                        actionTitle: Localized.SignMessage.viewFullMessage,
                        actionDestination: AnyView(TextMessageScene(model: model.textMessageViewModel)),
                    )
                    .presentationDetents([.large])
                    .presentationBackground(Colors.grayBackground)
                }
            }
        }
    }

    func sign() {
        model.onSign(onComplete: onComplete)
    }
}
