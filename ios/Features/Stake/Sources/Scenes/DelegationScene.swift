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
                ForEach(model.rows.filter { $0 != .rewards }, id: \.self) { row in
                    content(for: row)
                }
            }

            if model.rows.contains(.rewards) {
                Section {
                    content(for: .rewards)
                }
            }

            if model.showManage {
                Section(model.manageTitle) {
                    ForEach(model.availableActions) { action in
                        NavigationCustomLink(with: ListItemView(title: action.title)) {
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
            let item = ListItemView(title: model.title(for: row), subtitle: model.model.validatorText)
            if let url = model.providerUrl {
                SafariNavigationLink(url: url) { item }
            } else {
                item
            }
        case .apr:
            ListItemView(title: model.aprModel.title, subtitle: model.aprModel.subtitle)
        case .status:
            ListItemView(title: model.title(for: row), subtitle: model.stateModel.title, subtitleStyle: model.stateModel.textStyle)
        case .completionDate:
            ListItemView(title: model.title(for: row), subtitle: model.model.completionDateText)
        case .rewards:
            let rewardsItem = ListItemView(
                title: model.title(for: row),
                titleStyle: model.model.titleStyle,
                subtitle: model.model.rewardsText,
                subtitleStyle: model.model.subtitleStyle,
                subtitleExtra: model.model.rewardsFiatValueText,
                subtitleStyleExtra: model.model.subtitleExtraStyle,
                imageStyle: model.assetImageStyle,
            )
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
