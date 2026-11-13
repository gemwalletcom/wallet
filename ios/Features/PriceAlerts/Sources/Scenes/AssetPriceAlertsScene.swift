// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Localization
import Primitives
import PrimitivesComponents
import Store
import Style
import SwiftUI

public struct AssetPriceAlertsScene: View {
    @State private var model: AssetPriceAlertsSceneViewModel

    public init(model: AssetPriceAlertsSceneViewModel) {
        _model = State(initialValue: model)
    }

    public var body: some View {
        let assetAlerts = model.assetAlerts
        return List {
            if case let .error(error) = assetAlerts.list.phase {
                Section {
                    ListItemErrorView(errorTitle: Localized.Errors.errorOccurred, error: error)
                }
            }

            Section {
                Toggle(isOn: model.isAutoAlertEnabledBinding(assetAlerts)) {
                    ListAssetItemView(row: assetAlerts.autoRow.row)
                }
                .toggleStyle(AppToggleStyle())
            } footer: {
                Text(Localized.PriceAlerts.autoFooter)
            }

            GemGroupSections(groups: assetAlerts.list.groups) { item in
                if case let .priceAlert(alert) = item {
                    PriceAlertItemView(item: alert, onDelete: { onDelete(alert: $0) })
                }
            }

            if case let .empty(state) = assetAlerts.list.phase {
                EmptyContentView(model: EmptyStateViewModel(state: state))
                    .padding(.top, .extraLarge)
                    .cleanListRow()
            }
        }
        .bindQuery(model.query)
        .bindQuery(model.priceQuery)
        .listSectionSpacing(.compact)
        .refreshable { await model.load() }
        .task { await model.load() }
        .navigationTitle(model.title)
        .navigationBarTitleDisplayMode(.inline)
        .toolbar {
            ToolbarItem(placement: .navigationBarTrailing) {
                Button(action: model.onSelectSetPriceAlert) {
                    Image(systemName: SystemImage.plus)
                }
                .accessibilityIdentifier("add_price_alert")
            }
        }
        .sheet(isPresented: $model.isPresentingSetPriceAlert) {
            SetPriceAlertNavigationStack(model: model.setPriceAlertModel())
        }
        .toast(message: $model.isPresentingToastMessage)
    }
}

// MARK: - Actions

extension AssetPriceAlertsScene {
    private func onDelete(alert: PriceAlert) {
        Task {
            await model.deletePriceAlert(priceAlert: alert)
        }
    }
}
