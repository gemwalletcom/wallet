// Copyright (c). Gem Wallet. All rights reserved.

@testable import AppService
import BannerService
import DeviceServiceTestKit
import Foundation
import GemstonePrimitivesTestKit
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
    func setupSeedsWalletBannersAndSyncsConfiguration() async throws {
        let wallet = Wallet.mock(id: .multicoin(address: "0x\(UUID().uuidString)"), source: .create)
        let preferences = Preferences.mock()
        let db = DB.mockWithChains([.xrp, .stellar, .algorand])
        let bannerStore = BannerStore.mock(db: db)
        try WalletStore.mock(db: db).addWallet(wallet)
        let walletConfigurationService = GemWalletConfigurationServiceMock()
        let service = OnstartWalletService(
            deviceService: DeviceServiceMock(),
            bannerSetupService: BannerSetupService(store: bannerStore, preferences: preferences),
            walletConfigurationService: walletConfigurationService,
            pushNotificationEnablerService: PushNotificationEnablerService(preferences: preferences),
        )

        await service.setup(wallet: wallet).value

        #expect(try bannerStore.getBanner(id: "\(wallet.id.id)_\(BannerEvent.onboarding.rawValue)")?.event == .onboarding)
        #expect(await walletConfigurationService.walletIds == [wallet.id.id])
    }
}
