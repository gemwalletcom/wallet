// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Primitives
import PrimitivesComponents
import SwiftUI

public struct PerpetualPositionsList: View {
    private let positions: [PerpetualPositionData]
    private let onSelect: AssetAction
    @Binding private var showBalancePrivacy: Bool

    public init(
        positions: [PerpetualPositionData],
        showBalancePrivacy: Binding<Bool> = .constant(false),
        onSelect: AssetAction = nil,
    ) {
        self.positions = positions
        _showBalancePrivacy = showBalancePrivacy
        self.onSelect = onSelect
    }

    public var body: some View {
        ForEach(positions) { position in
            if let onSelect {
                NavigationCustomLink(
                    with: listItem(for: position),
                    action: { onSelect(position.perpetualData.asset) },
                )
            } else {
                NavigationLink(value: Scenes.Perpetual(position.perpetualData)) {
                    listItem(for: position)
                }
            }
        }
    }

    private func listItem(for position: PerpetualPositionData) -> ListAssetItemView {
        ListAssetItemView(
            model: PerpetualPositionItemViewModel(
                model: PerpetualPositionViewModel(position),
                showBalancePrivacy: $showBalancePrivacy,
            ),
        )
    }
}
