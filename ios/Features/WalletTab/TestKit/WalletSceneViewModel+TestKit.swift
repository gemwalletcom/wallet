// Copyright (c). Gem Wallet. All rights reserved.

import class Gemstone.GemWalletPreferencesService
import class Gemstone.GemNftService
import GemstoneServicesTestKit
import Foundation
import PreferencesTestKit
import Primitives
import PrimitivesTestKit
import GemstoneServices
import GemstonePrimitivesTestKit
import WalletTab

public extension WalletSceneViewModel {
    static func mock(wallet: Wallet = .mock()) -> WalletSceneViewModel {
        WalletSceneViewModel(
            assetDiscoveryService: GemAssetDiscoveryServiceMock(),
            balanceService: GemBalanceServiceMock(),
            bannerService: GemBannerServiceMock(),
            nftService: GemNftService.mock(),
            walletPreferencesService: GemWalletPreferencesService.mock(),
            observablePreferences: .mock(),
            wallet: wallet,
            isPresentingSelectedAssetInput: .constant(.none),
            isPresentingWallets: .constant(false),
        )
    }
}
