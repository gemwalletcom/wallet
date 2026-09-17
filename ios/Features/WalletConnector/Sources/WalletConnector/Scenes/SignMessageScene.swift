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
            if let headerModel = model.headerModel {
                TransactionHeaderListItemView(headerType: .assetValue(headerModel), showClearHeader: true)
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
                    SimulationWarningsContent(models: model.simulationWarningModels)
                }
            }

            if model.payloadModel.hasFields {
                Section {
                    SimulationPayloadFieldsContent(models: model.fieldModels(for: model.payloadModel.primaryFields))

                    NavigationCustomLink(with: ListItemView(model: model.payloadDetailsListItem)) {
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
                        primaryModels: model.fieldModels(for: model.payloadModel.primaryFields),
                        secondaryModels: model.fieldModels(for: model.payloadModel.secondaryFields),
                        actionListItem: model.viewFullMessageListItem,
                        actionDestination: AnyView(TextMessageScene(model: model.textMessageViewModel)),
                    )
                }
                .sheetPresentation([.large])
            }
        }
    }

    func sign() {
        model.onSign(onComplete: onComplete)
    }
}
