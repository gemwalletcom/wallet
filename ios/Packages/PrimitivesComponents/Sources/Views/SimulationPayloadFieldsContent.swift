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
            switch model.kind {
            case let .address(context): row(model).explorerContext(context)
            case .plain: row(model)
            }
        }
    }

    @ViewBuilder
    private func row(_ model: SimulationPayloadFieldViewModel) -> some View {
        if let onSelect = model.onSelect {
            NavigationCustomLink(with: ListItemView(model: model.listItem), action: onSelect)
        } else {
            ListItemView(model: model.listItem)
        }
    }
}
