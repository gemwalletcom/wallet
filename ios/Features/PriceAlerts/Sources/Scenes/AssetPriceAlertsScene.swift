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

            if model.alertsModel.isNotEmpty {
                Section {
                    ForEach(model.alertsModel, id: \.data.priceAlert.id) { alertModel in
                        alertView(model: alertModel)
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

    private func alertView(model: PriceAlertItemViewModel) -> some View {
        ListAssetItemView(model: model)
            .swipeActions(edge: .trailing) {
                Button(Localized.Common.delete, role: .destructive) {
                    onDelete(alert: model.data.priceAlert)
                }
                .tint(Colors.red)
            }
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
