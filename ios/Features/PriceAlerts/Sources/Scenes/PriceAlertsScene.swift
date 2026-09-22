// Copyright (c). Gem Wallet. All rights reserved.

import Components
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
        List {
            toggleView

            if let error = model.loadError {
                Section {
                    ListItemErrorView(errorTitle: Localized.Errors.errorOccurred, error: error)
                }
            }

            ListItemValueSectionList(
                list: model.sections,
                content: { alert in
                    NavigationLink(value: Scenes.Price(asset: alert.asset)) {
                        PriceAlertItemView(alert: alert, currency: model.currency, onDelete: { onDelete(alert: $0) })
                    }
                },
            )
        }
        .bindQuery(model.query)
        .contentMargins(.top, .scene.top, for: .scrollContent)
        .listSectionSpacing(.compact)
        .overlay {
            if model.priceAlerts.isEmpty, model.loadError == nil {
                EmptyContentView(model: model.emptyContentModel)
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
            Toggle(
                model.enableTitle,
                isOn: $model.isPriceAlertsEnabled,
            )
            .toggleStyle(AppToggleStyle())
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
