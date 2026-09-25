// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Localization
import Primitives
import PrimitivesComponents
import SwiftUI

public struct EarnScene: View {
    private let model: EarnSceneViewModel

    public init(model: EarnSceneViewModel) {
        self.model = model
    }

    public var body: some View {
        let earn = model.earnView
        List {
            ListAssetHeaderView(model: model.assetModel)

            switch model.providersState(earn) {
            case .noData:
                Section {
                    ListItemView(model: model.noDataListItem)
                }
            case .loading:
                ListItemLoadingView()
                    .id(UUID())
            case .data:
                Section {
                    GemListRowView(row: earn.aprRow)
                }
            case let .error(error):
                ListItemErrorView(errorTitle: Localized.Errors.errorOccurred, error: error)
            }

            if model.depositRoute(earn) != nil {
                Section(Localized.Common.manage) {
                    NavigationCustomLink(with: ListItemView(model: model.depositListItem)) {
                        model.onSelectDeposit()
                    }
                }
            }

            Section(model.positionsSectionTitle(earn)) {
                if earn.positions.isNotEmpty {
                    ForEach(earn.positions) { item in
                        NavigationCustomLink(with: DelegationView(delegation: DelegationViewModel(row: item.row))) {
                            model.onSelect(item: item)
                        }
                    }
                    .listRowInsets(.assetListRowInsets)
                } else if model.showsEmptyState(earn) {
                    EmptyContentView(model: model.emptyContentModel)
                        .cleanListRow()
                }
            }
        }
        .listSectionSpacing(.compact)
        .navigationTitle(model.title)
        .refreshable {
            await model.load()
        }
        .taskOnce {
            Task {
                await model.load()
            }
        }
    }
}
