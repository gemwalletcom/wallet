// Copyright (c). Gem Wallet. All rights reserved.

import Components
import struct Gemstone.GemStakeViewState
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
        let state = model.viewState
        List {
            headerSection
            stakeInfoSection(state)
            ForEach(model.sectionModels(state)) { section in
                Section(section.title) {
                    content(for: section, state: state)
                }
            }
            if model.showsDelegationsPlaceholder(state) {
                Section {
                    delegationsPlaceholder(state)
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
    private func content(for section: StakeSectionViewModel, state: GemStakeViewState) -> some View {
        switch section.section {
        case .manage:
            ForEach(model.actionModels(state)) { item in
                actionLink(item, state: state)
            }
        case .resources:
            ForEach(state.resourceRows, id: \.self) { row in
                GemListRowView(row: row)
            }
        case .delegations:
            delegationsPlaceholder(state)
        }
    }

    @ViewBuilder
    private func actionLink(_ item: StakeActionViewModel, state: GemStakeViewState) -> some View {
        if let infoAction = item.infoAction {
            NavigationCustomLink(with: ListItemView(model: item.model), action: infoAction)
        } else {
            NavigationCustomLink(with: ListItemView(model: item.model)) {
                model.onSelect(destination: item.destination, state: state)
            }
            .enabled(item.isEnabled)
        }
    }

    @ViewBuilder
    private func delegationsPlaceholder(_ state: GemStakeViewState) -> some View {
        switch model.delegationsViewState(state) {
        case .noData:
            EmptyContentView(model: model.emptyContentModel)
                .cleanListRow()
        case .loading:
            ListItemLoadingView()
                .id(UUID())
        case let .data(items):
            ForEach(items, id: \.id) { item in
                NavigationCustomLink(with: DelegationView(delegation: DelegationViewModel(row: item.row))) {
                    model.onSelect(delegation: item, state: state)
                }
            }
            .listRowInsets(.assetListRowInsets)
        case let .error(error):
            ListItemErrorView(errorTitle: Localized.Errors.errorOccurred, error: error)
        }
    }

    private func stakeInfoSection(_ state: GemStakeViewState) -> some View {
        Section {
            ForEach(state.infoRows, id: \.self) { row in
                GemListRowView(row: row, onInfo: model.onInfo)
            }
        }
    }
}
