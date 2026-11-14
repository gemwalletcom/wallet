// Copyright (c). Gem Wallet. All rights reserved.

import Components
import GemstonePrimitives
import InfoSheet
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
            GemGroupSections(rows: model.sections) { row in
                switch row.action() {
                case .priceAlerts:
                    NavigationLink(
                        value: Scenes.AssetPriceAlert(asset: model.asset),
                        label: { GemListRowView(row: row) },
                    )
                case .setPriceAlert:
                    NavigationCustomLink(with: GemListRowView(row: row)) {
                        model.onSelectSetPriceAlerts()
                    }
                default:
                    GemListRowView(row: row, onSelectAddress: model.onSelectContract, onInfo: model.onInfo)
                }
            }
        }
        .bindQuery(model.priceQuery)
        .task(id: model.currency) {
            await model.onChangeCurrency()
        }
        .onChange(of: model.priceData, model.onChangePriceData)
        .navigationTitle(model.title)
        .sheet(item: $model.isPresentingInfoSheet) {
            InfoSheetScene(sheet: $0)
        }
    }
}
