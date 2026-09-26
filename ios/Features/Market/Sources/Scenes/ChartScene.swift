// Copyright (c). Gem Wallet. All rights reserved.

import Components
import GemstonePrimitives
import InfoSheet
import Localization
import Primitives
import PrimitivesComponents
import Store
import SwiftUI

public struct ChartScene: View {
    @State private var model: ChartSceneViewModel

    public init(model: ChartSceneViewModel) {
        _model = State(initialValue: model)
    }

    public var body: some View {
        ChartListView(model: model) {
            ForEach(model.sections.listSections) { section in
                Section {
                    ForEach(section.values) { item in
                        switch item.row.action() {
                        case .priceAlerts:
                            NavigationLink(
                                value: Scenes.AssetPriceAlert(asset: model.asset),
                                label: { GemListRowView(row: item.row) },
                            )
                        case .setPriceAlert:
                            NavigationCustomLink(with: GemListRowView(row: item.row)) {
                                model.onSelectSetPriceAlerts()
                            }
                        default:
                            GemListRowView(row: item.row, onSelectAddress: model.onSelectContract, onInfo: model.onInfo)
                        }
                    }
                } header: {
                    if let title = section.title {
                        Text(title)
                    }
                } footer: {
                    if let footer = section.footer {
                        Text(footer)
                    }
                }
            }
        }
        .bindQuery(model.priceQuery)
        .task(id: model.currency) {
            await model.onChangeCurrency()
        }
        .task(id: model.priceData) {
            await model.updateSections()
        }
        .navigationTitle(model.title)
        .sheet(item: $model.isPresentingInfoSheet) {
            InfoSheetScene(sheet: $0)
        }
    }
}
