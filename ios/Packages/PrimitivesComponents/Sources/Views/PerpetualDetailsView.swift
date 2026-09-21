// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Foundation
import SwiftUI

public struct PerpetualDetailsView: View {
    private let model: PerpetualDetailsViewModel

    public init(model: PerpetualDetailsViewModel) {
        self.model = model
    }

    public var body: some View {
        ListSectionView(provider: model) { row in
            GemListRowView(row: row)
        }
        .toolbarDismissItem(type: .close, placement: .topBarLeading)
        .navigationTitle(model.title)
        .navigationBarTitleDisplayMode(.inline)
        .listSectionSpacing(.compact)
        .contentMargins([.top], .extraSmall, for: .scrollContent)
    }
}
