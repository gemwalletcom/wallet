// Copyright (c). Gem Wallet. All rights reserved.

import Components
import enum Gemstone.GemTransactionHeader
import Primitives
import Style
import SwiftUI

public struct TransactionHeaderListItemView: View {
    private let header: GemTransactionHeader
    private let action: TransactionHeaderActionHandler?

    public init(
        header: GemTransactionHeader,
        action: TransactionHeaderActionHandler? = nil,
    ) {
        self.header = header
        self.action = action
    }

    public var body: some View {
        switch header {
        case .swap:
            Section {
                // Swap row has two distinct tap regions; SwapAmountView wires Buttons internally.
                TransactionHeaderView(header: header, action: action)
            }
        case .amount, .value, .nft, .assetImage:
            Section {
                headerRow.cleanListRow()
            }
        }
    }

    @ViewBuilder
    private var headerRow: some View {
        if let action {
            Button { action(.header) } label: {
                TransactionHeaderView(header: header)
            }
        } else {
            TransactionHeaderView(header: header)
        }
    }
}
