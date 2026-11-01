// Copyright (c). Gem Wallet. All rights reserved.

import class Gemstone.GemTransactionDetailsService
import enum Gemstone.GemTransactionHeaderAction
import protocol Gemstone.GemTransactionsServiceProtocol
import GemstonePrimitivesTestKit
import Primitives
import PrimitivesTestKit
import Transactions

public extension TransactionSceneViewModel {
    @MainActor
    static func mock(
        id: TransactionId = .mock(),
        type: TransactionType = .transfer,
        state: TransactionState = .confirmed,
        direction: TransactionDirection = .outgoing,
        asset: Asset = .mock(),
        swapToAsset: Asset? = nil,
        from: String = "",
        to: String = "participant_address",
        memo: String? = nil,
        metadata: AnyCodableValue? = nil,
        confirmationEtaSeconds: UInt32? = nil,
        isStored: Bool = true,
        wallet: Wallet = .mock(),
        transactionsService: any GemTransactionsServiceProtocol = GemTransactionsServiceMock(),
        onHeaderAction: ((GemTransactionHeaderAction) -> Void)? = nil,
    ) -> TransactionSceneViewModel {
        let swapMetadata = swapToAsset.flatMap {
            AnyCodableValue.encode(
                TransactionSwapMetadata.mock(
                    fromAsset: asset.id,
                    fromValue: "1000000000000000000",
                    toAsset: $0.id,
                    toValue: "200",
                    provider: SwapProvider.nearIntents.rawValue,
                ),
            )
        }
        let transaction = TransactionExtended.mock(
            transaction: .mock(
                id: id,
                assetId: asset.id,
                from: from,
                to: to,
                type: type,
                state: state,
                memo: memo,
                direction: direction,
                metadata: metadata ?? swapMetadata,
            ),
            asset: asset,
            assets: swapToAsset.map { [asset, $0] } ?? [],
            confirmationEtaSeconds: confirmationEtaSeconds,
        )
        return TransactionSceneViewModel(
            transactionId: id,
            transaction: isStored ? transaction : nil,
            wallet: wallet,
            service: GemTransactionDetailsService.mock(),
            transactionsService: transactionsService,
            onHeaderAction: onHeaderAction,
        )
    }
}
