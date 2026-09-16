// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Primitives
import SwiftUI

public struct SimulationPayloadFieldsContent: View {
    private let models: [SimulationPayloadFieldViewModel]

    public init(models: [SimulationPayloadFieldViewModel]) {
        self.models = models
    }

    public var body: some View {
        ForEach(Array(models.enumerated()), id: \.offset) {
            let model = $0.element
            ListItemView(title: model.title, subtitle: model.subtitle)
                .contextMenu(model.contextMenuItems)
        }
    }
}
