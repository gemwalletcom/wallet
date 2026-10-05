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
                    primaryRows: model.primaryPayloadFields,
                    secondaryRows: model.secondaryPayloadFields,
                    onSelectAddress: model.onSelectPayloadAddress,
                    actionListItem: model.viewFullMessageListItem,
                    actionDestination: AnyView(TextMessageScene(text: model.messageText)),
                )
            }
            .sheetPresentation([.large])
        }
    }
}
