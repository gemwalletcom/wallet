// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Primitives
import SwiftUI

public struct SimulationWarningsContent: View {
    private let warnings: [SimulationWarning]

    public init(warnings: [SimulationWarning]) {
        self.warnings = warnings
    }

    public var body: some View {
        ForEach(warnings.map(SimulationWarningViewModel.init).filter(\.isVisible)) {
            ListItemErrorView(
                errorTitle: $0.title,
                errorImageColor: $0.color,
                error: AnyError($0.message),
            )
        }
    }
}
