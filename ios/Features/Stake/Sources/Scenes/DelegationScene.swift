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
                ForEach(model.rows, id: \.self) { row in
                    GemListRowView(row: row)
                }
            }

            if let rewardsItem = model.rewardsItem {
                Section {
                    if model.canClaimRewards {
                        NavigationCustomLink(with: ListItemView(model: rewardsItem)) {
                            model.onClaimRewards()
                        }
                    } else {
                        ListItemView(model: rewardsItem)
                    }
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
}
