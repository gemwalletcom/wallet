// Copyright (c). Gem Wallet. All rights reserved.

import Components
import SwiftUI

public struct ChartListView<Model: ChartListViewable, Content: View>: View {
    @Environment(\.connectionStatus) private var connectionStatus

    let model: Model
    @ViewBuilder let content: () -> Content

    public init(model: Model, @ViewBuilder content: @escaping () -> Content) {
        self.model = model
        self.content = content
    }

    public var body: some View {
        List {
            Section {} header: {
                ChartStateView(model: model)
            }
            .fullWidthSection()
            content()
        }
        .listSectionSpacing(.compact)
        .background {
            ChartPeriodLoader(model: model)
        }
        .refreshable {
            await model.load()
        }
        .refreshableTimer(every: connectionStatus.refreshInterval(for: .chart)) { @MainActor _ in
            await model.load()
        }
    }
}

private struct ChartPeriodLoader<Model: ChartListViewable>: View {
    let model: Model

    var body: some View {
        Color.clear
            .task(id: model.selectedPeriod) {
                await model.load()
            }
    }
}
