// Copyright (c). Gem Wallet. All rights reserved.

import Components
import GemstonePrimitives
import Localization
import Primitives
import PrimitivesComponents
import Store
import Style
import SwiftUI

public struct PriceAlertsScene: View {
    @State private var model: PriceAlertsSceneViewModel

    public init(model: PriceAlertsSceneViewModel) {
        _model = State(initialValue: model)
    }

    public var body: some View {
        let list = model.list
        return List {
            toggleView

            if case let .error(error) = list.phase {
                Section {
                    ListItemErrorView(errorTitle: Localized.Errors.errorOccurred, error: error)
                }
            }

            ForEach(model.sections(list)) { section in
                Section {
                    ForEach(section.values) { item in
                        NavigationLink(value: model.chart(item)) {
                            PriceAlertItemView(item: item, onDelete: { onDelete(alert: $0) })
                        }
                        .assetIdentifier(item.data.asset.toPrimitives().id)
                    }
                } header: {
                    if let title = section.title, !title.isEmpty {
                        Text(title)
                    }
                } footer: {
                    if let footer = section.footer, !section.values.isEmpty {
                        Text(footer)
                    }
                }
                .listRowInsets(.assetListRowInsets)
            }
        }
        .bindQuery(model.query)
        .contentMargins(.top, .scene.top, for: .scrollContent)
        .listSectionSpacing(.compact)
        .overlay {
            if case let .empty(state) = list.phase {
                EmptyContentView(model: EmptyStateViewModel(state: state))
            }
        }
        .onChange(of: model.isPriceAlertsEnabled, onAlertsEnable)
        .refreshable {
            await model.load()
        }
        .task {
            await model.load()
        }
        .navigationTitle(model.title)
        .alertSheet($model.isPresentingAlertMessage)
    }
}

// MARK: - UI

private extension PriceAlertsScene {
    var toggleView: some View {
        Section {
            GemListRowView(row: model.toggleRow, onToggle: model.onToggle)
        } footer: {
            Text(Localized.PriceAlerts.getNotifiedExplainMessage)
        }
    }
}

// MARK: - Actions

extension PriceAlertsScene {
    func onDelete(alert: PriceAlert) {
        Task {
            await model.deletePriceAlert(priceAlert: alert)
        }
    }

    func onAlertsEnable(_ _: Bool, newValue: Bool) {
        Task {
            await model.setAlertsEnabled(newValue)
        }
    }
}
