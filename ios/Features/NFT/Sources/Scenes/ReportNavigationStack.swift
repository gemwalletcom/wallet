// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Primitives
import SwiftUI

public struct ReportNavigationStack: View {
    @State private var model: ReportNftSceneViewModel

    public init(model: ReportNftSceneViewModel) {
        _model = State(initialValue: model)
    }

    public var body: some View {
        NavigationStack {
            ReportSelectReasonScene(model: model)
                .toolbarDismissItem(type: .close, placement: .topBarLeading)
                .activityIndicator(isLoading: model.state.isLoading, message: model.progressMessage)
        }
        .sheetPresentation(.forCurrentDeviceSize())
    }
}
