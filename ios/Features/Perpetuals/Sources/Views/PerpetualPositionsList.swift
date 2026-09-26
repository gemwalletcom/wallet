// Copyright (c). Gem Wallet. All rights reserved.

import Components
import func Gemstone.perpetualPositionRows
import GemstonePrimitives
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
        ForEach(Array(zip(positions, perpetualPositionRows(positions: positions.map { $0.toGem() }))), id: \.1.id) { position, row in
            let itemView = ListAssetItemView(row: row.row, isPrivacyEnabled: $showBalancePrivacy)
            if let onSelect {
                NavigationCustomLink(
                    with: itemView,
                    action: { onSelect(position.perpetualData.asset) },
                )
            } else {
                NavigationLink(value: Scenes.Perpetual(position.perpetualData)) {
                    itemView
                }
            }
        }
    }
}
