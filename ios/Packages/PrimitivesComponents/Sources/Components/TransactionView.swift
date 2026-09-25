// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Primitives
import Style
import SwiftUI

public struct TransactionView: View {
    private let model: TransactionViewModel

    public init(model: TransactionViewModel) {
        self.model = model
    }

    public var body: some View {
        ListItemView(model: model.listItem)
    }
}

// MARK: - Previews

#Preview {
    let pendingTransactionMock = Transaction(
        id: TransactionId(chain: .smartChain, hash: "0xe5fb66cef0fb71fa75e0245484a40d17952cf46053724c6ac61209bf307d6e56"),
        assetId: .init(chain: .smartChain, tokenId: ""),
        from: "0x92abCE21234D71EC443E679f3a1feAFD3Fc830fB",
        to: "0x8d7460E51bCf4eD26877cb77E56f3ce7E9f5EB8F",
        contract: nil,
        type: .transfer,
        state: .pending,
        blockNumber: "39348339",
        sequence: "1",
        fee: "21000000000000",
        feeAssetId: .init(chain: .smartChain, tokenId: ""),
        value: "100000000000000",
        memo: nil,
        direction: .outgoing,
        utxoInputs: [],
        utxoOutputs: [],
        metadata: nil,
        createdAt: Date(),
    )
    let pendingTransactionListItemMock = TransactionListItem(
        transaction: pendingTransactionMock,
        asset: Primitives.Chain.smartChain.asset,
        assets: [],
        fromAddress: AddressName(chain: .smartChain, address: "0x92abCE21234D71EC443E679f3a1feAFD3Fc830fB", name: "test1", type: .address, status: .verified, imageUrl: nil),
        toAddress: AddressName(chain: .smartChain, address: "0x8d7460E51bCf4eD26877cb77E56f3ce7E9f5EB8F", name: "test2", type: .address, status: .verified, imageUrl: nil),
    )

    TransactionView(model: TransactionViewModel(transaction: pendingTransactionListItemMock))
}
