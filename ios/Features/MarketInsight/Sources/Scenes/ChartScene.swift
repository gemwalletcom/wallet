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
            ForEach(model.sections, id: \.self) { section in
                switch section {
                case let .priceAlerts(count):
                    Section {
                        NavigationLink(
                            value: Scenes.AssetPriceAlert(asset: model.asset),
                            label: {
                                ListItemView(model: model.listItem(for: section))
                            },
                        )
                    }
                case .setPriceAlert:
                    Section {
                        NavigationCustomLink(with: ListItemView(model: model.listItem(for: section))) {
                            model.onSelectSetPriceAlerts()
                        }
                    }
                case let .market(rows):
                    marketSection(model.marketValues(rows))
                case let .links(links):
                    Section(section.title ?? "") {
                        SocialLinksView(model: model.socialLinksModel(links))
                    }
                }
            }
        }
        .bindQuery(model.priceQuery)
        .navigationTitle(model.title)
        .sheet(item: $model.isPresentingInfoSheet) {
            InfoSheetScene(type: $0)
        }
    }

    private func marketSection(_ items: [MarketValueViewModel]) -> some View {
        Section {
            ForEach(items, id: \.title) { item in
                switch item.action {
                case let .explorer(explorerContext):
                    SafariNavigationLink(url: explorerContext.explorerLink.url) {
                        ListItemView(model: item.listItem())
                    }
                    .explorerContext(explorerContext)
                case let .info(type):
                    marketItemView(item, infoAction: { model.onSelectInfoSheet(type) })
                case .none:
                    marketItemView(item)
                }
            }
        }
    }

    private func marketItemView(_ item: MarketValueViewModel, infoAction: (() -> Void)? = nil) -> some View {
        ListItemView(model: item.listItem(infoAction: infoAction))
    }
}
