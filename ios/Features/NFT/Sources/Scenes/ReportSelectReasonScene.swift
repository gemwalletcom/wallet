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
                        with: ListItemView(title: reason.title),
                        action: { model.submitReport(reason: reason.rawValue) },
                    )
                }
            }
        }
        .navigationTitle(model.title)
        .navigationBarTitleDisplayMode(.inline)
    }
}
