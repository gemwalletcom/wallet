// Copyright (c). Gem Wallet. All rights reserved.

import PrimitivesComponents
import SwiftUI

public struct SignMessagePayloadDetails: View {
    private let model: SignMessageSceneViewModel

    public init(model: SignMessageSceneViewModel) {
        self.model = model
    }

    public var body: some View {
        if model.payloadModel.hasFields {
            NavigationStack {
                SimulationPayloadDetailsScene(
                    primaryModels: model.fieldModels(for: model.payloadModel.primaryFields),
                    secondaryModels: model.fieldModels(for: model.payloadModel.secondaryFields),
                    actionListItem: model.viewFullMessageListItem,
                    actionDestination: AnyView(TextMessageScene(text: model.messageText)),
                )
            }
            .sheetPresentation([.large])
        }
    }
}
