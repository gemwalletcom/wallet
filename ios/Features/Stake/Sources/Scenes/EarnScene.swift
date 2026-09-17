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
        List {
            ListAssetHeaderView(model: model.assetModel)

            switch model.providersState {
            case .noData:
                Section {
                    ListItemView(model: model.noDataListItem)
                }
            case .loading:
                ListItemLoadingView()
                    .id(UUID())
            case .data:
                Section {
                    ListItemView(model: model.aprListItem)
                }
            case let .error(error):
                ListItemErrorView(errorTitle: Localized.Errors.errorOccurred, error: error)
            }

            if model.showDeposit {
                Section(Localized.Common.manage) {
                    NavigationLink(value: model.depositDestination) {
                        ListItemView(model: model.depositListItem)
                    }
                }
            }

            Section(model.positionsSectionTitle) {
                if model.hasPositions {
                    ForEach(model.positionModels) { delegation in
                        NavigationLink(value: model.navigationDestination(for: delegation)) {
                            DelegationView(delegation: delegation)
                        }
                    }
                    .listRowInsets(.assetListRowInsets)
                } else if model.showEmptyState {
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
