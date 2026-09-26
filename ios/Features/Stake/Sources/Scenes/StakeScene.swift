// Copyright (c). Gem Wallet. All rights reserved.

import Components
import struct Gemstone.GemStakeActionItem
import enum Gemstone.GemStakeSection
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
            ListAssetHeaderView(model: state.asset)
            stakeInfoSection(state)
            ForEach(state.sections) { section in
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
        .ifLet(state.docsUrl?.asURL) { view, url in
            view.toolbarInfoButton(url: url)
        }
        .taskOnce {
            Task {
                await model.load()
            }
        }
    }
}

// MARK: - UI Components

extension StakeScene {
    @ViewBuilder
    private func content(for section: GemStakeSection, state: GemStakeViewState) -> some View {
        switch section {
        case .manage:
            ForEach(state.actions, id: \.action) { item in
                actionLink(item)
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
    private func actionLink(_ item: GemStakeActionItem) -> some View {
        switch item.tap {
        case .frozenBalanceInfo:
            NavigationCustomLink(with: GemListRowView(row: item.row, onInfo: { _ in model.onStakeFrozenInfo() }), action: model.onStakeFrozenInfo)
        case .disabled:
            NavigationCustomLink(with: GemListRowView(row: item.row)) {}
                .enabled(false)
        case let .open(destination):
            NavigationCustomLink(with: GemListRowView(row: item.row)) {
                model.onSelect(destination: destination)
            }
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
                    model.onSelect(delegation: item)
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
