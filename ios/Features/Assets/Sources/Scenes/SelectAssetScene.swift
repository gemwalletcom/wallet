import Components
import GemstonePrimitives
import Localization
import Primitives
import PrimitivesComponents
import Style
import SwiftUI

public struct SelectAssetScene: View {
    @State private var model: SelectAssetSceneViewModel

    public init(
        model: SelectAssetSceneViewModel,
    ) {
        _model = State(wrappedValue: model)
    }

    public var body: some View {
        let sections = model.sections
        let listState = model.listState(sections)
        return list(sections)
            .searchable(
                text: $model.searchableQuery,
                placement: .navigationBarDrawer(displayMode: .always),
            )
            .if(model.isNetworkSearchEnabled) {
                $0.debounce(
                    value: $model.searchableQuery.wrappedValue,
                    interval: GemConstants.searchDebounce,
                    action: model.search(query:),
                )
            }
            .overlay {
                if listState == .loading {
                    LoadingView()
                } else if listState != .idle {
                    EmptyContentView(model: model.emptyModel)
                }
            }
            .bindQuery(model.assetsQuery, model.recentModel.query)
            .onChange(of: model.searchableQuery, model.updateRequest)
            .copyToast($model.copyToast)
            .toast(message: $model.isPresentingToastMessage)
            .navigationBarTitle(model.title)
    }

    func list(_ sections: AssetsSections) -> some View {
        let assetItems = model.assetItems
        return List {
            if model.showRecents {
                RecentAssetsSectionView(
                    model: model.recentModel,
                    onSelect: model.onSelectRecent,
                )
            }

            ForEach(sections.sections, id: \.kind) { section in
                Section {
                    assetsList(assets: section.assets, assetItems: assetItems)
                } header: {
                    if let title = section.kind.title {
                        SectionHeaderView(title: title, image: section.kind.image)
                    } else {
                        Text(model.assetsTitle)
                    }
                }
                .listRowInsets(.assetListRowInsets)
            }
        }
        .contentMargins([.top], .extraSmall, for: .scrollContent)
        .listSectionSpacing(.compact)
    }

    func assetsList(assets: [AssetData], assetItems: ListAssetItemsViewModel) -> some View {
        let rows = assetItems.rows(assets.map(model.displayAssetData))
        return ForEach(Array(zip(assets, rows)), id: \.0.id) { assetData, row in
            let itemView = ListAssetItemView(row: row) { model.onAssetAction(action: $0, assetData: assetData) }
            switch model.flow.rowAction {
            case .navigate:
                NavigationCustomLink(with: itemView) {
                    model.onSelectAsset(assetData)
                }
            case .toggle:
                itemView
            case .select:
                NavigationCustomLink(with: itemView) {
                    model.selectAsset(asset: assetData.asset)
                }
            }
        }
    }
}
