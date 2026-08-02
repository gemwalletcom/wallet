// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Primitives
import Style
import SwiftUI

public struct TransactionHeaderListItemView: View {
    private let headerType: TransactionHeaderType
    private let showClearHeader: Bool
    private let action: TransactionHeaderActionHandler?
    private let hideBalance: Bool

    public init(
        headerType: TransactionHeaderType,
        showClearHeader: Bool,
        action: TransactionHeaderActionHandler? = nil,
        hideBalance: Bool = false,
    ) {
        self.headerType = headerType
        self.showClearHeader = showClearHeader
        self.action = action
        self.hideBalance = hideBalance
    }

    public init(
        model: TransactionHeaderItemModel,
        action: TransactionHeaderActionHandler? = nil,
        hideBalance: Bool = false,
    ) {
        headerType = model.headerType
        showClearHeader = model.showClearHeader
        self.action = action
        self.hideBalance = hideBalance
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
            TransactionHeaderView(type: headerType, action: action, hideBalance: hideBalance)
        case .amount, .nft, .asset, .assetValue:
            if let action {
                Button { action(.header) } label: {
                    TransactionHeaderView(type: headerType, hideBalance: hideBalance)
                }
            } else {
                TransactionHeaderView(type: headerType, hideBalance: hideBalance)
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
