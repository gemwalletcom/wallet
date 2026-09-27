// Copyright (c). Gem Wallet. All rights reserved.

import Components
import PrimitivesComponents
import Store
import SwiftUI

public struct DelegationScene: View {
    @State private var model: DelegationSceneViewModel

    public init(model: DelegationSceneViewModel) {
        _model = State(initialValue: model)
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
                    ForEach(details.actions, id: \.action) { item in
                        NavigationCustomLink(with: ListItemView(model: model.actionListItem(item))) {
                            model.onSelectAction(item)
                        }
                    }
                }
            }
        }
        .bindQuery(model.validatorsQuery)
        .navigationTitle(details.title.text)
        .listSectionSpacing(.compact)
    }
}
