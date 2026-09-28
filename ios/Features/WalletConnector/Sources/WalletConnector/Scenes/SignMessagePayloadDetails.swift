// Copyright (c). Gem Wallet. All rights reserved.

import PrimitivesComponents
import SwiftUI

public struct SignMessagePayloadDetails: View {
    private let model: SignMessageSceneViewModel

    public init(model: SignMessageSceneViewModel) {
        self.model = model
    }

    public var body: some View {
        if model.hasPayloadFields {
            NavigationStack {
                SimulationPayloadDetailsScene(
                    primaryModels: model.fieldModels(for: model.primaryPayloadFields),
                    secondaryModels: model.fieldModels(for: model.secondaryPayloadFields),
                    actionListItem: model.viewFullMessageListItem,
                    actionDestination: AnyView(TextMessageScene(text: model.messageText)),
                )
            }
            .sheetPresentation([.large])
        }
    }
}
