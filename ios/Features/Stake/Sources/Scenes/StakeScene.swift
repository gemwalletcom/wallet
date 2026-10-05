// Copyright (c). Gem Wallet. All rights reserved.

import Components
import enum Gemstone.GemListPhase
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
            if state.delegationsPhase != .rows {
                Section {
                    delegationsPlaceholder(state.delegationsPhase)
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
            ForEach(state.delegations, id: \.id) { item in
                NavigationCustomLink(with: ListItemView(model: item.row.listItem)) {
                    model.onSelect(delegation: item)
                }
            }
            .listRowInsets(.assetListRowInsets)
        }
    }

    @ViewBuilder
    private func actionLink(_ item: GemStakeActionItem) -> some View {
        switch item.action {
        case .frozenBalanceInfo:
            NavigationCustomLink(with: GemListRowView(row: item.row, onInfo: { _ in model.onStakeFrozenInfo() }), action: model.onStakeFrozenInfo)
        case .disabled:
            NavigationCustomLink(with: GemListRowView(row: item.row)) {}
                .enabled(false)
        case let .open(destination):
            NavigationCustomLink(with: GemListRowView(row: item.row)) {
                model.onSelect(kind: item.kind, destination: destination)
            }
        }
    }

    @ViewBuilder
    private func delegationsPlaceholder(_ phase: GemListPhase?) -> some View {
        switch phase {
        case let .empty(state):
            EmptyContentView(model: model.emptyContentModel(state))
                .cleanListRow()
        case .none:
            ListItemLoadingView()
                .id(UUID())
        case let .error(error):
            ListItemErrorView(errorTitle: Localized.Errors.errorOccurred, error: error)
        case .rows:
            EmptyView()
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
