// Copyright (c). Gem Wallet. All rights reserved.

import Formatters
import Foundation
import GemstonePrimitives
import Primitives
import Store

@Observable
@MainActor
final class PerpetualsPreviewViewModel {
    private let currencyFormatter: CurrencyFormatter

    let positionsQuery: ObservableQuery<PerpetualPositionsRequest>
    let walletBalanceQuery: ObservableQuery<PerpetualWalletBalanceRequest>

    var positions: [PerpetualPositionData] {
        positionsQuery.value
    }

    var walletBalance: WalletBalance {
        walletBalanceQuery.value
    }

    init(walletId: WalletId, currencyFormatter: CurrencyFormatter = .usd) {
        self.currencyFormatter = currencyFormatter
        positionsQuery = ObservableQuery(PerpetualPositionsRequest(walletId: walletId), initialValue: [])
        walletBalanceQuery = ObservableQuery(
            PerpetualWalletBalanceRequest(walletId: walletId, assetId: Chain.hyperCore.defaultAsset(type: .perpetual).id),
            initialValue: .zero,
        )
    }

    var tradePerpetualsSubtitle: String {
        currencyFormatter.string(walletBalance.total)
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
