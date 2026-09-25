// Copyright (c). Gem Wallet. All rights reserved.

import Components
import enum Gemstone.GemListRow
import Primitives
import PrimitivesComponents
import Style
import SwiftUI

public struct NotificationsScene: View {
    @State private var model: NotificationsViewModel

    public init(model: NotificationsViewModel) {
        _model = State(initialValue: model)
    }

    public var body: some View {
        ListSectionView(sections: model.sections) { row in
            content(for: row)
        }
        .contentMargins(.top, .scene.top, for: .scrollContent)
        .listSectionSpacing(.compact)
        .onChange(of: model.isEnabled) { _, newValue in
            Task { await model.enable(isEnabled: newValue) }
        }
        .alertSheet($model.isPresentingAlertMessage)
        .navigationTitle(model.title)
    }
}

// MARK: - UI Components

extension NotificationsScene {
    @ViewBuilder
    private func content(for row: GemListRow) -> some View {
        if case .link(.priceAlerts, _, _) = row {
            NavigationLink(value: Scenes.PriceAlerts()) {
                GemListRowView(row: row)
            }
        } else {
            GemListRowView(row: row, onToggle: model.onToggle)
        }
    }
}
