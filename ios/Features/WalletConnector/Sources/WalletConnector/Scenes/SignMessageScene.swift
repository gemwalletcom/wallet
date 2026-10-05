// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Localization
import Primitives
import PrimitivesComponents
import Style
import SwiftUI

public struct SignMessageScene: View {
    @Bindable var model: SignMessageSceneViewModel
    private let onComplete: () -> Void

    public init(
        model: SignMessageSceneViewModel,
        onComplete: @escaping () -> Void,
    ) {
        self.model = model
        self.onComplete = onComplete
    }

    public var body: some View {
        List {
            if let headerModel = model.headerModel {
                TransactionHeaderListItemView(header: .value(header: headerModel))
            } else {
                ListAssetHeaderView(model: model.appPreview, subtitleLayout: .vertical)
            }

            Section {
                ForEach(model.rows, id: \.self) { GemListRowView(row: $0) }
            }

            if model.hasWarnings {
                Section {
                    ForEach(model.simulationWarnings, id: \.self) { GemListRowView(row: $0) }
                }
            }

            if model.hasPayloadFields {
                Section {
                    ForEach(Array(model.primaryPayloadFields.enumerated()), id: \.offset) {
                        GemListRowView(row: $0.element, onSelectAddress: model.onSelectPayloadAddress)
                    }

                    if !model.secondaryPayloadFields.isEmpty {
                        NavigationCustomLink(with: ListItemView(model: model.payloadDetailsListItem)) {
                            model.onViewPayloadDetails()
                        }
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
    }

    func sign() {
        model.onSign(onComplete: onComplete)
    }
}
