// Copyright (c). Gem Wallet. All rights reserved.

import Components
import enum Gemstone.GemDelegationRow
import PrimitivesComponents
import SwiftUI

public struct DelegationScene: View {
    private let model: DelegationSceneViewModel

    public init(model: DelegationSceneViewModel) {
        self.model = model
    }

    public var body: some View {
        List {
            Section {} header: {
                ValueHeaderView(
                    model: model.model,
                    isPrivacyEnabled: .constant(false),
                    titleActionType: .none,
                    onHeaderAction: nil,
                    onInfoAction: nil,
                )
                .padding(.top, .small)
            }
            .cleanListRow()

            Section {
                ForEach(model.detailRows, id: \.self) { row in
                    content(for: row)
                }
            }

            if let rewardsRow = model.rewardsRow {
                Section {
                    content(for: rewardsRow)
                }
            }

            if model.showManage {
                Section(model.manageTitle) {
                    ForEach(model.availableActions) { action in
                        NavigationCustomLink(with: ListItemView(model: model.actionListItem(action))) {
                            model.onSelectAction(action)
                        }
                    }
                }
            }
        }
        .navigationTitle(model.title)
        .listSectionSpacing(.compact)
    }

    @ViewBuilder
    private func content(for row: GemDelegationRow) -> some View {
        switch row {
        case .provider:
            let item = ListItemView(model: model.listItem(for: row))
            if let url = model.providerUrl {
                SafariNavigationLink(url: url) { item }
            } else {
                item
            }
        case .apr:
            ListItemView(model: model.listItem(for: row))
        case .status:
            ListItemView(model: model.listItem(for: row))
        case .completionDate:
            ListItemView(model: model.listItem(for: row))
        case .rewards:
            let rewardsItem = ListItemView(model: model.listItem(for: row))
            if model.canClaimRewards {
                NavigationCustomLink(with: rewardsItem) {
                    model.onClaimRewards()
                }
            } else {
                rewardsItem
            }
        }
    }
}
