// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Primitives
import SwiftUI
import struct Gemstone.GemSimulationWarningRow

public struct SimulationWarningsContent: View {
    private let warnings: [GemSimulationWarningRow]

    public init(warnings: [GemSimulationWarningRow]) {
        self.warnings = warnings
    }

    public var body: some View {
        ForEach(warnings.map(SimulationWarningViewModel.init)) {
            ListItemErrorView(
                errorTitle: $0.title,
                errorImageColor: $0.color,
                error: AnyError($0.message),
            )
        }
    }
}
