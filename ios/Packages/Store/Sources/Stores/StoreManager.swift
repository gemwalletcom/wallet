// Copyright (c). Gem Wallet. All rights reserved.

import Foundation

public struct StoreManager: Sendable {
    public let assetStore: AssetStore
    public let balanceStore: BalanceStore
    public let fiatRateStore: FiatRateStore
    public let priceStore: PriceStore
    public let transactionStore: TransactionStore
    public let nodeStore: NodeStore
    public let walletStore: WalletStore
    public let connectionsStore: ConnectionsStore
    public let stakeStore: StakeStore
    public let bannerStore: BannerStore
    public let priceAlertStore: PriceAlertStore
    public let nftStore: NFTStore
    public let addressStore: AddressStore
    public let perpetualStore: PerpetualStore
    public let recentActivityStore: RecentActivityStore
    public let searchStore: SearchStore
    public let inAppNotificationStore: InAppNotificationStore
    public let contactStore: ContactStore
    public let fiatTransactionStore: FiatTransactionStore

    public init(db: DB) {
        assetStore = AssetStore(db: db)
        balanceStore = BalanceStore(db: db)
        fiatRateStore = FiatRateStore(db: db)
        priceStore = PriceStore(db: db)
        transactionStore = TransactionStore(db: db)
        nodeStore = NodeStore(db: db)
        walletStore = WalletStore(db: db)
        connectionsStore = ConnectionsStore(db: db)
        stakeStore = StakeStore(db: db)
        bannerStore = BannerStore(db: db)
        priceAlertStore = PriceAlertStore(db: db)
        nftStore = NFTStore(db: db)
        addressStore = AddressStore(db: db)
        perpetualStore = PerpetualStore(db: db)
        recentActivityStore = RecentActivityStore(db: db)
        searchStore = SearchStore(db: db)
        inAppNotificationStore = InAppNotificationStore(db: db)
        contactStore = ContactStore(db: db)
        fiatTransactionStore = FiatTransactionStore(db: db)
    }
}
