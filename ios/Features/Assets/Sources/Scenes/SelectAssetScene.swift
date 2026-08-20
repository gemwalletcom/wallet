import Components
import Localization
import Primitives
import PrimitivesComponents
import Recents
import Style
import SwiftUI

public struct SelectAssetScene: View {
    @State private var model: SelectAssetViewModel

    public init(
        model: SelectAssetViewModel,
    ) {
        _model = State(wrappedValue: model)
    }

    public var body: some View {
        list
        .searchable(
            text: $model.searchableQuery,
            placement: .navigationBarDrawer(displayMode: .always),
        )
        .if(model.isNetworkSearchEnabled) {
            $0.debounce(
                value: $model.searchableQuery.wrappedValue,
                action: model.search(query:),
            )
        }
        .overlay {
            if model.showLoading {
                LoadingView()
            } else if model.showEmpty {
                EmptyContentView(
                    model: EmptyContentTypeViewModel(
                        type: .search(
                            type: .assets,
                            action: model.showAddToken ? { model.onSelectAddCustomToken() } : nil,
                        ),
                    ),
                )
            }
        }
        .bindQuery(model.assetsQuery, model.recentModel.query)
        .onChange(of: model.filterModel, model.onChangeFilterModel)
        .onChange(of: model.searchableQuery, model.updateRequest)
        .ifLet(model.copyTypeViewModel) {
            $0.copyToast(
                model: $1,
                isPresenting: $model.isPresentingCopyToast,
            )
        }
        .navigationBarTitle(model.title)
    }

    var list: some View {
        List {
            if model.showRecents {
                RecentAssetsSectionView(
                    model: model.recentModel,
                    onSelect: model.onSelectRecent,
                )
            }

            if model.showPopularSection {
                Section {
                    assetsList(assets: model.sections.popular)
                } header: {
                    HStack {
                        model.popularImage
                        Text(model.popularTitle)
                    }
                }
                .listRowInsets(.assetListRowInsets)
            }

            if model.showPinnedSection {
                Section {
                    assetsList(assets: model.sections.pinned)
                } header: {
                    PinnedSectionHeader()
                }
                .listRowInsets(.assetListRowInsets)
            }

            if model.showAssetsSection {
                Section {
                    assetsList(assets: model.sections.assets)
                } header: {
                    Text(model.assetsTitle)
                }
                .listRowInsets(.assetListRowInsets)
            }
        }
        .contentMargins([.top], .extraSmall, for: .scrollContent)
        .listSectionSpacing(.compact)
    }

    func assetsList(assets: [AssetData]) -> some View {
        ForEach(assets) { assetData in
            let itemView = ListAssetItemSelectionView(
                assetData: model.displayAssetData(assetData),
                currencyCode: model.currencyCode,
                type: model.flow.listType,
                action: model.onAssetAction,
            )
            switch model.flow.rowSelection {
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
