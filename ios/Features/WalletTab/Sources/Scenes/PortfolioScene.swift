// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Localization
import Primitives
import PrimitivesComponents
import Style
import SwiftUI

public struct PortfolioScene: View {
    @Bindable private var model: PortfolioSceneViewModel

    public init(model: PortfolioSceneViewModel) {
        self.model = model
    }

    public var body: some View {
        NavigationStack {
            ChartListView(model: model) {
                if model.statisticRows.isNotEmpty {
                    Section {
                        ForEach(Array(model.statisticRows.enumerated()), id: \.offset) { _, row in
                            GemListRowView(row: row)
                        }
                    } header: {
                        Text(model.statisticsTitle)
                    }
                }
            }
            .task(id: model.selectedType) { await model.loadIfNeeded() }
            .navigationTitle(model.navigationTitle)
            .navigationBarTitleDisplayMode(.inline)
            .toolbarDismissItem(type: .close, placement: .cancellationAction)
            .toolbar {
                if model.showSegmentedControl {
                    ToolbarItem(placement: .principal) {
                        Picker("", selection: $model.selectedType) {
                            ForEach(PortfolioType.allCases) { type in
                                Text(model.typeTitle(for: type)).tag(type)
                            }
                        }
                        .pickerStyle(.segmented)
                        .fixedSize()
                    }
                }
                if model.showChartTypePicker {
                    ToolbarItem(placement: .topBarTrailing) {
                        Menu {
                            Picker("", selection: $model.selectedChartType) {
                                ForEach(PortfolioChartType.allCases) { type in
                                    Text(model.chartTypeTitle(for: type)).tag(type)
                                }
                            }
                        } label: {
                            Text(model.chartTypeTitle(for: model.selectedChartType))
                                .fontWeight(.semibold)
                        }
                    }
                }
            }
        }
    }
}
