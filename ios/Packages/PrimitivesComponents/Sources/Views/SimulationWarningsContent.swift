// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Primitives
import SwiftUI

public struct SimulationWarningsContent: View {
    private let models: [SimulationWarningViewModel]

    public init(models: [SimulationWarningViewModel]) {
        self.models = models
    }

    public var body: some View {
        ForEach(models) {
            ListItemErrorView(
                errorTitle: $0.title,
                errorImageColor: $0.color,
                error: AnyError($0.message),
            )
        }
    }
}
