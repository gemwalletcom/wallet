// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Primitives
import Style
import SwiftUI

public struct TransactionHeaderListItemView: View {
    private let headerType: TransactionHeaderType
    private let showClearHeader: Bool
    private let action: TransactionHeaderActionHandler?

    public init(
        headerType: TransactionHeaderType,
        showClearHeader: Bool,
        action: TransactionHeaderActionHandler? = nil,
    ) {
        self.headerType = headerType
        self.showClearHeader = showClearHeader
        self.action = action
    }

    public init(
        model: TransactionHeaderItemModel,
        action: TransactionHeaderActionHandler? = nil,
    ) {
        headerType = model.headerType
        showClearHeader = model.showClearHeader
        self.action = action
    }

    public var body: some View {
        if showClearHeader {
            Section {
                headerRow.cleanListRow()
            }
        } else {
            Section {
                headerRow
            }
        }
    }

    @ViewBuilder
    private var headerRow: some View {
        switch headerType {
        case .swap:
            // Swap row has two distinct tap regions; SwapAmountView wires Buttons internally.
            TransactionHeaderView(type: headerType, action: action)
        case .amount, .nft, .asset, .assetValue:
            if let action {
                Button { action(.header) } label: {
                    TransactionHeaderView(type: headerType)
                }
            } else {
                TransactionHeaderView(type: headerType)
            }
        }
    }
}

#Preview {
    List {
        TransactionHeaderListItemView(
            headerType:
            .swap(
                from: .init(
                    assetId: AssetId(chain: .abstract, tokenId: nil),
                    assetImage: .image(Images.Chains.abstract),
                    amount: "300",
                    fiatAmount: "300$",
                ),
                to: .init(
                    assetId: AssetId(chain: .arbitrum, tokenId: nil),
                    assetImage: .image(Images.Chains.arbitrum),
                    amount: "200",
                    fiatAmount: "200$",
                ),
            ),
            showClearHeader: true,
        )
    }
}
