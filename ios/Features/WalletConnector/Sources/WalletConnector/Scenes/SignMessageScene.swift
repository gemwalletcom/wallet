// Copyright (c). Gem Wallet. All rights reserved.

import Components
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

            SignMessagePayloadSection(
                primaryRows: model.primaryPayloadFields,
                secondaryRows: model.secondaryPayloadFields,
                messageText: model.messageText,
                onSelectAddress: model.onSelectPayloadAddress,
                onViewDetails: model.onViewPayloadDetails,
            )
        }
        .contentMargins(.top, .scene.top, for: .scrollContent)
        .listSectionSpacing(.compact)
        .taskOnce { model.load() }
        .safeAreaButton {
            StateButton(
                text: model.buttonTitle,
                state: model.buttonState,
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
