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
            ListAssetHeaderView(model: model.appPreview, subtitleLayout: .vertical)

            Section {
                if let merchant = model.merchantText {
                    ListItemImageView(
                        title: model.merchantTitle,
                        subtitle: merchant,
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

            if let expiresAt = model.expiresAt {
                Section {
                    ListItemExpiryView(
                        title: model.expiresTitle,
                        expiresAt: expiresAt,
                    )
                }
            }

            if let fee = model.networkFeeText {
                Section {
                    ListItemView(title: model.networkFeeTitle, subtitle: fee)
                }
            }

            if model.hasWarnings {
                Section {
                    SimulationWarningsContent(warnings: model.simulationWarnings)
                }
            }

            if model.showsPayload {
                Section {
                    SimulationPayloadFieldsContent(
                        fields: model.primaryPayloadFields,
                        fieldViewModel: model.payloadFieldViewModel(for:),
                        contextMenuItems: model.contextMenuItems(for:),
                    )

                    NavigationCustomLink(with: ListItemView(title: Localized.Common.details)) {
                        model.onViewPayloadDetails()
                    }
                }
            } else if case let .text(string) = model.messageDisplayType {
                Section(Localized.SignMessage.message) {
                    Text(string)
                }
            }
        }
        .contentMargins(.top, .scene.top, for: .scrollContent)
        .listSectionSpacing(.compact)
        .taskOnce { model.fetch() }
        .task { await model.paymentExpiry.start() }
        .safeAreaButton {
            StateButton(
                text: model.buttonTitle,
                type: model.buttonType,
                action: sign,
            )
        }
        .navigationTitle(model.title)
        .safariSheet(url: $model.isPresentingUrl)
        .sheet(isPresented: $model.isPresentingPayloadDetails) {
            if model.hasPayload {
                NavigationStack {
                    SimulationPayloadDetailsScene(
                        primaryFields: model.primaryPayloadFields,
                        secondaryFields: model.secondaryPayloadFields,
                        fieldViewModel: model.payloadFieldViewModel(for:),
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
        Task {
            do {
                try await model.signMessage()
                onComplete()
            } catch {
                debugLog("sign message error \(error)")
            }
        }
    }
}
