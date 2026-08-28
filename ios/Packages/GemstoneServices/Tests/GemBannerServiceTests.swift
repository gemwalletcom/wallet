// Copyright (c). Gem Wallet. All rights reserved.

import class Gemstone.GemBannerService
import Foundation
import GemstonePrimitives
@testable import GemstoneServices
import GemstoneServicesTestKit
import Primitives
import PrimitivesTestKit
import Store
import StoreTestKit
import Testing

struct GemBannerServiceTests {
    @Test
    func closeActionCancelsBanner() async throws {
        let (store, banner, service) = try makeService()

        try await service.applyAction(key: banner.gemKey, action: .close)

        #expect(try store.getBanner(id: banner.id)?.state == .cancelled)
    }

    @Test
    func setupWalletSeedsOnboardingForCreatedWalletsOnly() async throws {
        let created = Wallet.mock(id: .multicoin(address: "0xcreated"), source: .create)
        let imported = Wallet.mock(id: .multicoin(address: "0ximported"), source: .import)
        let (store, _, service) = try makeService(chains: [.xrp, .stellar, .algorand], wallets: [created, imported])

        try await service.setupWallet(wallet: created.json())
        try await service.setupWallet(wallet: imported.json())

        #expect(try store.getBanner(id: "\(created.id.id)_onboarding")?.state == .active)
        #expect(try store.getBanner(id: "\(imported.id.id)_onboarding") == nil)
        #expect(try store.getBanner(id: "xrp_accountActivation")?.state == .active)
    }

    private func makeService(
        banner: Banner = Banner(walletId: nil, asset: nil, event: .stake, state: .active),
        chains: [Chain] = [],
        wallets: [Wallet] = [],
    ) throws -> (BannerStore, Banner, GemBannerService) {
        let db = DB.mockWithChains(chains)
        let walletStore = WalletStore.mock(db: db)
        for wallet in wallets {
            try walletStore.addWallet(wallet)
        }
        let store = BannerStore.mock(db: db)
        try store.addBanners([NewBanner(walletId: banner.walletId?.id, assetId: banner.asset?.id, event: banner.event, state: banner.state)])
        let service = GemBannerService(store: GemstoneBannerStore(store: store))
        return (store, banner, service)
    }
}
