// Copyright (c). Gem Wallet. All rights reserved.

import Components
import enum Gemstone.GemStakeSection
import struct Gemstone.GemStakeActionItem
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
            ForEach(model.sections, id: \.self) { section in
                Section(section.title) {
                    content(for: section)
                }
            }
            if !model.sections.contains(.delegations) {
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
    private func content(for section: GemStakeSection) -> some View {
        switch section {
        case .manage:
            ForEach(model.actions, id: \.action) { item in
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
    private func actionLink(_ item: GemStakeActionItem) -> some View {
        if let infoAction = model.frozenBalanceInfoAction(for: item) {
            NavigationCustomLink(with: ListItemView(model: model.actionListItem(item)), action: infoAction)
        } else {
            NavigationLink(value: model.destination(for: item.action)) {
                ListItemView(model: model.actionListItem(item))
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
                NavigationLink(value: model.navigationDestination(for: delegation)) {
                    DelegationView(delegation: delegation)
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
                ListItemView(
                    field: model.infoField(for: row),
                    infoAction: model.infoAction(for: row),
                )
            }
        }
    }
}
