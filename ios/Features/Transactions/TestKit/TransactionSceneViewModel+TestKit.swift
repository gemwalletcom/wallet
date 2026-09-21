// Copyright (c). Gem Wallet. All rights reserved.

import class Gemstone.GemTransactionDetailsService
import enum Gemstone.GemTransactionHeaderAction
import GemstonePrimitivesTestKit
import Primitives
import PrimitivesTestKit
import Transactions

public extension TransactionSceneViewModel {
    @MainActor
    static func mock(
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
        wallet: Wallet = .mock(),
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
        return TransactionSceneViewModel(
            transaction: .mock(
                transaction: .mock(
                    type: type,
                    state: state,
                    direction: direction,
                    assetId: asset.id,
                    from: from,
                    to: to,
                    memo: memo,
                    metadata: metadata ?? swapMetadata,
                ),
                asset: asset,
                assets: swapToAsset.map { [asset, $0] } ?? [],
                confirmationEtaSeconds: confirmationEtaSeconds,
            ),
            wallet: wallet,
            service: GemTransactionDetailsService.mock(),
            onHeaderAction: onHeaderAction,
        )
    }
}
