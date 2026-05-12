// Copyright (c). Gem Wallet. All rights reserved.

@testable import AppService
import BannerService
import DeviceServiceTestKit
import Foundation
import GemAPITestKit
import NotificationService
import Preferences
import PreferencesTestKit
import Primitives
import PrimitivesTestKit
import Store
import StoreTestKit
import Testing

struct OnstartWalletServiceTests {
    @Test
    func setupAddsMultiSignatureBanner() async throws {
        let walletId = WalletId.multicoin(address: "0x\(UUID().uuidString)")
        let walletPreferences = WalletPreferences(walletId: walletId)
        defer { walletPreferences.clear() }

        let wallet = Wallet.mock(
            id: walletId,
            accounts: [.mock(chain: .tron, address: "tron-address")],
        )
        let preferences = Preferences.mock()
        let db = DB.mockWithChains([.tron, .xrp, .stellar, .algorand])
        let bannerStore = BannerStore.mock(db: db)
        try WalletStore.mock(db: db).addWallet(wallet)
        let walletConfigurationService = GemAPIWalletConfigurationServiceMock(
            result: WalletConfigurationResult(
                walletId: walletId,
                configuration: WalletConfiguration(
                    multiSignatureAccounts: [ChainAddress(chain: .tron, address: "tron-address")],
                ),
            ),
        )
        let service = OnstartWalletService(
            deviceService: DeviceServiceMock(),
            bannerSetupService: BannerSetupService(store: bannerStore, preferences: preferences),
            walletConfigurationService: walletConfigurationService,
            pushNotificationEnablerService: PushNotificationEnablerService(preferences: preferences),
        )

        await service.setup(wallet: wallet).value

        let banner = try bannerStore.getBanner(id: "\(walletId.id)_\(Chain.tron.id)_\(BannerEvent.accountBlockedMultiSignature.rawValue)")
        #expect(banner?.event == .accountBlockedMultiSignature)
        #expect(banner?.state == .alwaysActive)
        #expect(walletPreferences.completeInitialWalletConfiguration)
        #expect(await walletConfigurationService.walletIds == [walletId])
    }

    @Test
    func setupSkipsBannerWhenAlreadySynchronized() async throws {
        let walletId = WalletId.multicoin(address: "0x\(UUID().uuidString)")
        let walletPreferences = WalletPreferences(walletId: walletId)
        defer { walletPreferences.clear() }
        walletPreferences.completeInitialWalletConfiguration = true

        let wallet = Wallet.mock(id: walletId, accounts: [.mock(chain: .tron, address: "tron-address")])
        let preferences = Preferences.mock()
        let db = DB.mockWithChains([.tron])
        let bannerStore = BannerStore.mock(db: db)
        try WalletStore.mock(db: db).addWallet(wallet)
        let walletConfigurationService = GemAPIWalletConfigurationServiceMock()
        let service = OnstartWalletService(
            deviceService: DeviceServiceMock(),
            bannerSetupService: BannerSetupService(store: bannerStore, preferences: preferences),
            walletConfigurationService: walletConfigurationService,
            pushNotificationEnablerService: PushNotificationEnablerService(preferences: preferences),
        )

        await service.setup(wallet: wallet).value

        #expect(await walletConfigurationService.walletIds.isEmpty)
    }
}
