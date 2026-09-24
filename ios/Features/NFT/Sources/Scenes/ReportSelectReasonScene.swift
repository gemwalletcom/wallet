// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Primitives
import PrimitivesComponents
import SwiftUI

struct ReportSelectReasonScene: View {
    @Bindable private var model: ReportNftViewModel

    init(model: ReportNftViewModel) {
        self.model = model
    }

    var body: some View {
        List {
            Section {
                ForEach(model.reasons, id: \.self) { reason in
                    NavigationCustomLink(
                        with: ListItemView(model: model.listItem(for: reason)),
                        action: { onSelectReason(reason) },
                    )
                }
            }
        }
        .navigationTitle(model.title)
        .navigationBarTitleDisplayMode(.inline)
        .alertSheet($model.isPresentingAlertMessage)
    }
}

// MARK: - Actions

extension ReportSelectReasonScene {
    private func onSelectReason(_ reason: ReportReason) {
        Task { await model.submitReport(reason: reason) }
    }
}
