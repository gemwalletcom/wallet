// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Primitives
import SwiftUI

struct ReportSelectReasonScene: View {
    private let model: ReportNftViewModel

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
    }
}

// MARK: - Actions

extension ReportSelectReasonScene {
    private func onSelectReason(_ reason: ReportReason) {
        Task { await model.submitReport(reason: reason.rawValue) }
    }
}
