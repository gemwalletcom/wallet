// Copyright (c). Gem Wallet. All rights reserved.

import Components
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
                ForEach(model.detailRowModels) { row in
                    content(for: row)
                }
            }

            if let rewardsRow = model.rewardsRowModel {
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
    private func content(for row: DelegationRowViewModel) -> some View {
        let item = ListItemView(model: row.model)
        switch row.action {
        case .plain:
            item
        case let .url(url):
            SafariNavigationLink(url: url) { item }
        case .claimRewards:
            NavigationCustomLink(with: item) {
                model.onClaimRewards()
            }
        }
    }
}
