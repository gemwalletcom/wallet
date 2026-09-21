// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import func Gemstone.perpetualBalanceTotal
import GemstonePrimitives
import Localization
import Primitives
import Store

@Observable
@MainActor
final class PerpetualsPreviewViewModel {
    let positionsQuery: ObservableQuery<PerpetualPositionsRequest>
    let walletBalanceQuery: ObservableQuery<PerpetualWalletBalanceRequest>

    var positions: [PerpetualPositionData] {
        positionsQuery.value
    }

    init(walletId: WalletId) {
        positionsQuery = ObservableQuery(PerpetualPositionsRequest(walletId: walletId), initialValue: [])
        walletBalanceQuery = ObservableQuery(
            PerpetualWalletBalanceRequest(walletId: walletId, assetId: Chain.hyperCore.defaultAsset(type: .perpetual).id),
            initialValue: nil,
        )
    }

    var tradePerpetualsTitle: String {
        Localized.Perpetuals.trade
    }

    var tradePerpetualsSubtitle: String {
        perpetualBalanceTotal(balance: walletBalanceQuery.value?.balance.toGem()).text()
    }

    var hasNoPositions: Bool {
        positions.isEmpty
    }

    func updateWallet(walletId: WalletId) {
        positionsQuery.request = PerpetualPositionsRequest(walletId: walletId)
        walletBalanceQuery.request = PerpetualWalletBalanceRequest(
            walletId: walletId,
            assetId: Chain.hyperCore.defaultAsset(type: .perpetual).id,
        )
    }
}
