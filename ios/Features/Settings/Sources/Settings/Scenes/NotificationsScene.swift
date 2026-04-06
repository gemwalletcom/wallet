// Copyright (c). Gem Wallet. All rights reserved.

import Components
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
        List {
            Section {
                Toggle(
                    model.title,
                    isOn: $model.isEnabled,
                )
                .toggleStyle(AppToggleStyle())
            }

            Section {
                NavigationLink(value: Scenes.PriceAlerts()) {
                    ListItemView(
                        title: model.priceAlertsTitle,
                        imageStyle: .settings(assetImage: model.priceAlertsImage),
                    )
                }
            }
        }
        .contentMargins(.top, .scene.top, for: .scrollContent)
        .listSectionSpacing(.compact)
        .onChange(of: model.isEnabled) { _, newValue in
            Task {
                try await model.enable(isEnabled: newValue)
            }
        }
        .navigationTitle(model.title)
    }
}
