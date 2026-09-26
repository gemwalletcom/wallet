// Copyright (c). Gem Wallet. All rights reserved.

import Primitives
import PrimitivesTestKit
import Store
import StoreTestKit
import Testing

struct BannersQueryTests {
    private let db = DB.mock()
    private let wallet = Wallet.mock()
    private let otherWallet = Wallet.mock(id: .multicoin(address: "other-wallet"))

    init() throws {
        try AssetStore(db: db).add(assets: [
            .mock(asset: .mock(id: .mock(chain: .tron), name: "TRON", symbol: "TRX", decimals: 6)),
            .mock(asset: .mock(id: .mock(chain: .tron, tokenId: "TR7NHqjeKQxGTCi8q8ZY4pL8otSzgjLj6t"), name: "Tether USD", symbol: "USDT", decimals: 6, type: .trc20)),
            .mock(asset: .mock(id: .mock(chain: .ethereum), name: "Ethereum", symbol: "ETH", decimals: 18)),
        ])
        try WalletStore(db: db).addWallet(wallet)
        try WalletStore(db: db).addWallet(otherWallet)
        try BannerStore(db: db).addBanners([
            NewBanner(id: "tron-warning", walletId: wallet.id.id, assetId: Chain.tron.assetId, event: .accountBlockedMultiSignature, state: .alwaysActive),
        ])
    }

    @Test(arguments: [
        Asset.mock(id: .mock(chain: .tron), name: "TRON", symbol: "TRX", decimals: 6).id,
        Asset.mock(id: .mock(chain: .tron, tokenId: "TR7NHqjeKQxGTCi8q8ZY4pL8otSzgjLj6t"), name: "Tether USD", symbol: "USDT", decimals: 6, type: .trc20).id,
    ])
    func nativeAndTokenAssetsIncludeMultiSignatureWarning(assetId: AssetId) throws {
        let banners = try db.dbQueue.read {
            try BannersQuery(walletId: wallet.id, assetId: assetId, events: BannerEvent.allCases).fetch($0)
        }

        #expect(banners.map(\.event) == [.accountBlockedMultiSignature])
        #expect(banners.first?.asset == .mock(id: .mock(chain: .tron), name: "TRON", symbol: "TRX", decimals: 6))
    }

    @Test
    func tokenLoadsCandidatesForTheSameWalletAndChain() throws {
        try BannerStore(db: db).addBanners([
            NewBanner(id: "other-wallet", walletId: otherWallet.id.id, assetId: Chain.tron.assetId, event: .accountBlockedMultiSignature, state: .alwaysActive),
            NewBanner(id: "other-chain", walletId: wallet.id.id, assetId: Chain.ethereum.assetId, event: .accountBlockedMultiSignature, state: .alwaysActive),
            NewBanner(id: "stake", assetId: Chain.tron.assetId, event: .stake, state: .active),
        ])
        let banners = try db.dbQueue.read {
            try BannersQuery(walletId: wallet.id, assetId: Asset.mock(id: .mock(chain: .tron, tokenId: "TR7NHqjeKQxGTCi8q8ZY4pL8otSzgjLj6t"), name: "Tether USD", symbol: "USDT", decimals: 6, type: .trc20).id, events: BannerEvent.allCases)
                .fetch($0)
        }

        #expect(banners.count == 2)
        #expect(Set(banners.map(\.event)) == [.accountBlockedMultiSignature, .stake])
        #expect(banners.first { $0.event == .accountBlockedMultiSignature }?.walletId == wallet.id)
        #expect(banners.allSatisfy { $0.asset == .mock(id: .mock(chain: .tron), name: "TRON", symbol: "TRX", decimals: 6) })
    }

    @Test
    func cancelledWarningKeepsItsStateForCore() throws {
        try BannerStore(db: db).updateState("tron-warning", state: .cancelled)
        let banners = try db.dbQueue.read {
            try BannersQuery(walletId: wallet.id, assetId: Asset.mock(id: .mock(chain: .tron, tokenId: "TR7NHqjeKQxGTCi8q8ZY4pL8otSzgjLj6t"), name: "Tether USD", symbol: "USDT", decimals: 6, type: .trc20).id, events: BannerEvent.allCases)
                .fetch($0)
        }

        #expect(banners.map(\.state) == [.cancelled])
    }
}
