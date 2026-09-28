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
        let alerts = model.alerts(assetAlerts)
        return List {
            if let error = model.loadError {
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

            if alerts.isNotEmpty {
                Section {
                    ForEach(alerts) { item in
                        PriceAlertItemView(item: item, onDelete: { onDelete(alert: $0) })
                    }
                } header: {
                    Text(Localized.Stake.active)
                }
            }

            if model.showsEmpty(assetAlerts) {
                EmptyContentView(model: model.emptyContentModel)
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
