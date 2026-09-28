// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Primitives
import SwiftUI

struct ReportNavigationStack: View {
    @State private var model: ReportNftSceneViewModel

    init(model: ReportNftSceneViewModel) {
        _model = State(initialValue: model)
    }

    var body: some View {
        NavigationStack {
            ReportSelectReasonScene(model: model)
                .toolbarDismissItem(type: .close, placement: .topBarLeading)
                .activityIndicator(isLoading: model.state.isLoading, message: model.progressMessage)
        }
        .sheetPresentation(.forCurrentDeviceSize())
    }
}
