// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Localization
import Primitives
import PrimitivesComponents
import Store
import SwiftUI

public struct StakeScene: View {
    private let model: StakeSceneViewModel

    public init(model: StakeSceneViewModel) {
        self.model = model
    }

    public var body: some View {
        List {
            headerSection
            stakeInfoSection
            ForEach(model.sectionModels) { section in
                Section(section.title) {
                    content(for: section)
                }
            }
            if model.showsDelegationsPlaceholder {
                Section {
                    delegationsPlaceholder
                }
            }
        }
        .listSectionSpacing(.compact)
        .refreshable {
            await model.load()
        }
        .navigationTitle(model.title)
        .taskOnce {
            Task {
                await model.load()
            }
        }
    }
}

// MARK: - UI Components

extension StakeScene {
    private var headerSection: some View {
        ListAssetHeaderView(model: model.assetModel)
    }

    @ViewBuilder
    private func content(for section: StakeSectionViewModel) -> some View {
        switch section.section {
        case .manage:
            ForEach(model.actionModels) { item in
                actionLink(item)
            }
        case .resources:
            ListItemView(field: model.energyField)
            ListItemView(field: model.bandwidthField)
        case .delegations:
            delegationsPlaceholder
        }
    }

    @ViewBuilder
    private func actionLink(_ item: StakeActionViewModel) -> some View {
        if let infoAction = item.infoAction {
            NavigationCustomLink(with: ListItemView(model: item.model), action: infoAction)
        } else {
            NavigationCustomLink(with: ListItemView(model: item.model)) {
                model.onSelect(destination: item.destination)
            }
            .enabled(item.isEnabled)
        }
    }

    @ViewBuilder
    private var delegationsPlaceholder: some View {
        switch model.delegationsViewState {
        case .noData:
            EmptyContentView(model: model.emptyContentModel)
                .cleanListRow()
        case .loading:
            ListItemLoadingView()
                .id(UUID())
        case let .data(delegations):
            ForEach(delegations) { delegation in
                NavigationCustomLink(with: DelegationView(delegation: delegation)) {
                    model.onSelect(delegation: delegation)
                }
            }
            .listRowInsets(.assetListRowInsets)
        case let .error(error):
            ListItemErrorView(errorTitle: Localized.Errors.errorOccurred, error: error)
        }
    }

    private var stakeInfoSection: some View {
        Section {
            ForEach(model.infoRows, id: \.self) { row in
                GemListRowView(row: row, onInfo: model.onInfo)
            }
        }
    }
}
