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
                                ListItemView(title: Localized.Settings.PriceAlerts.title, subtitle: "\(count)")
                            },
                        )
                    }
                case .setPriceAlert:
                    Section {
                        NavigationCustomLink(with: ListItemView(title: Localized.PriceAlerts.SetAlert.title)) {
                            model.onSelectSetPriceAlerts()
                        }
                    }
                case let .market(rows):
                    marketSection(model.marketValues(rows))
                case let .links(links):
                    Section(Localized.Social.links) {
                        SocialLinksView(model: SocialLinksViewModel(assetLinks: links.map { $0.map() }))
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
                        ListItemView(title: item.title, subtitle: item.subtitle)
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
        ListItemView(
            title: item.title,
            titleTag: item.titleTag,
            titleTagStyle: item.titleTagStyle ?? .body,
            titleExtra: item.titleExtra,
            subtitle: item.subtitle,
            subtitleExtra: item.subtitleExtra,
            subtitleStyleExtra: item.subtitleExtraStyle ?? .calloutSecondary,
            infoAction: infoAction,
        )
    }
}
