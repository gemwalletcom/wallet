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
        let details = model.details
        List {
            Section {} header: {
                ValueHeaderView(
                    header: model.header(details),
                    isPrivacyEnabled: .constant(false),
                    titleActionType: .none,
                    onHeaderAction: nil,
                    onInfoAction: nil,
                )
                .padding(.top, .small)
            }
            .cleanListRow()

            Section {
                ForEach(details.rows, id: \.self) { row in
                    GemListRowView(row: row, onSelectAddress: model.onSelectProvider)
                }
            }

            if let rewardsItem = model.rewardsItem(details) {
                Section {
                    if let claim = details.claim {
                        NavigationCustomLink(with: ListItemView(model: rewardsItem)) {
                            model.onClaimRewards(claim)
                        }
                    } else {
                        ListItemView(model: rewardsItem)
                    }
                }
            }

            if details.actions.isNotEmpty {
                Section(model.manageTitle) {
                    ForEach(details.actions) { action in
                        NavigationCustomLink(with: ListItemView(model: model.actionListItem(action))) {
                            model.onSelectAction(action)
                        }
                    }
                }
            }
        }
        .navigationTitle(details.title.text)
        .listSectionSpacing(.compact)
    }
}
