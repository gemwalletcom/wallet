// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Localization
import Primitives
import PrimitivesComponents
import Store
import Style
import SwiftUI

public struct AssetPriceAlertsScene: View {
    @State private var model: AssetPriceAlertsViewModel

    public init(model: AssetPriceAlertsViewModel) {
        _model = State(initialValue: model)
    }

    public var body: some View {
        List {
            Section {
                Toggle(isOn: model.isAutoAlertEnabledBinding) {
                    ListAssetItemView(model: model.autoAlertItemModel)
                }
                .toggleStyle(AppToggleStyle())
            } footer: {
                Text(Localized.PriceAlerts.autoFooter)
            }

            if model.alerts.isNotEmpty {
                Section {
                    ForEach(model.alerts, id: \.priceAlert.id) { alert in
                        PriceAlertItemView(alert: alert, currency: model.currency, onDelete: { onDelete(alert: $0) })
                    }
                } header: {
                    Text(Localized.Stake.active)
                }
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
